// Pas de console noire sous Windows en version release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{ContextMenu, MenuBuilder},
    webview::{Color, NewWindowResponse, PageLoadEvent, WebviewBuilder},
    window::WindowBuilder,
    AppHandle, LogicalPosition, LogicalSize, Manager, State, Theme, Url, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

/// Largeur (en pixels logiques) de la barre latérale
const SIDEBAR_W: f64 = 68.0;
/// Fond de la barre latérale (`--bg`), repris par la fenêtre
const BG: Color = Color(0x28, 0x2a, 0x36, 0xff);
/// Fond de la zone de contenu (`--bg-dark`), repris par les vues de comptes avant leur premier affichage
const CONTENT_BG: Color = Color(0x21, 0x22, 0x2c, 0xff);
const GMAIL: &str = "https://mail.google.com/mail/u/0/";

#[derive(Serialize, Deserialize, Clone)]
struct Account {
    id: String,
    name: String,
    /// Adresse et avatar du compte Google connecté, signalés par la vue du compte
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
    /// Couleur de fond choisie (`#rrggbb`) ; à défaut, la barre en déduit une de l'identifiant
    #[serde(default)]
    color: Option<String>,
}

impl Account {
    fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            email: None,
            avatar: None,
            color: None,
        }
    }
}

/// Injecté dans les vues de comptes : repère l'adresse et l'avatar du compte Google connecté
/// (bouton de compte en haut à droite, à défaut le titre de l'onglet) et les signale au backend.
const PROFILE_SCRIPT: &str = r#"
(() => {
  if (window.top !== window || !/(^|\.)google\.com$/.test(location.hostname)) return;
  const EMAIL = /[\w.+-]+@[\w-]+(\.[\w-]+)+/;
  const AVATAR = /^https:\/\/([\w.-]+\.(googleusercontent|ggpht)\.com|lh3\.google\.com)\//;
  // Photo dans une image (src ou srcset) ou, selon les pages, en image de fond
  const avatarIn = (root) => {
    if (!root) return null;
    for (const img of root.querySelectorAll("img")) {
      const src = img.currentSrc || img.src;
      if (AVATAR.test(src)) return src;
    }
    for (const el of [root, ...root.querySelectorAll("[style*='background']")]) {
      const m = getComputedStyle(el).backgroundImage.match(/url\("?([^")]+)"?\)/);
      if (m && AVATAR.test(m[1])) return m[1];
    }
    return null;
  };
  // La photo est transmise en data: URL, stockée avec le compte : la barre latérale l'affiche
  // sans réseau et sans adresse qui expire. Les cookies du compte, absents de la barre, sont
  // indispensables pour une photo privée (lh3.google.com), sinon Google renvoie une lettre.
  const toDataUrl = async (url) => {
    if (!url) return null;
    const res = await fetch(url.replace(/=s\d+/, "=s96"), { credentials: "include" });
    if (!res.ok) throw new Error(res.status);
    const blob = await res.blob();
    return new Promise((ok, ko) => {
      const r = new FileReader();
      r.onload = () => ok(r.result);
      r.onerror = ko;
      r.readAsDataURL(blob);
    });
  };
  let last = "";
  const check = () => {
    const btn = document.querySelector('a[href*="accounts.google.com/SignOutOptions"]');
    const label = (btn && btn.getAttribute("aria-label")) || "";
    const email = (label.match(EMAIL) || document.title.match(EMAIL) || [])[0];
    // À défaut du lien habituel, le bouton de compte est l'élément dont le libellé cite l'adresse
    const avatar = email && (avatarIn(btn) || avatarIn(document.querySelector('[aria-label*="' + email + '"]')));
    const key = email + "|" + avatar;
    if (email && key !== last) {
      last = key;
      // Photo illisible : on ne réessaie qu'au prochain changement d'adresse
      toDataUrl(avatar)
        .catch(() => null)
        .then((avatar) => window.__TAURI_INTERNALS__.invoke("report_profile", { email, avatar }))
        .catch(() => { last = ""; });
    }
    // Toutes les 3 s tant que le profil n'est pas trouvé, puis seulement toutes les minutes
    // (la photo peut changer après coup). Sans photo, on attend encore 30 s qu'elle s'affiche.
    if (email && !avatar) missing++;
    setTimeout(check, email && (avatar || missing > 10) ? 60000 : 3000);
  };
  let missing = 0;
  setTimeout(check, 3000);
})();
"#;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Store {
    accounts: Vec<Account>,
    active: Option<String>,
    /// Thème clair (Alucard) au lieu du thème sombre (Dracula), choisi dans la barre latérale
    #[serde(default)]
    light: bool,
}

/// Thème de la barre de titre native, assorti à celui de l'application
fn window_theme(light: bool) -> Option<Theme> {
    Some(if light { Theme::Light } else { Theme::Dark })
}

struct AppState {
    store: Mutex<Store>,
    dir: PathBuf,
    /// Sérialise la fermeture et la création des vues de comptes (voir `show_active`)
    views: Mutex<()>,
    /// Numéro de la dernière vue de compte créée : le filet de sécurité d'une vue remplacée
    /// entre-temps ne doit pas afficher la nouvelle avant la fin de son chargement.
    generation: AtomicU64,
}

impl AppState {
    /// Écrit dans un fichier temporaire puis le renomme : une coupure pendant l'écriture ne
    /// laisse jamais un `accounts.json` tronqué.
    fn save(&self, store: &Store) {
        if let Ok(json) = serde_json::to_string_pretty(store) {
            let tmp = self.dir.join("accounts.json.tmp");
            if fs::write(&tmp, json).is_ok() {
                let _ = fs::rename(&tmp, self.dir.join("accounts.json"));
            }
        }
    }
}

/// Charge les comptes. Le booléen indique si la liste est fiable : faux si `accounts.json`
/// existe mais est illisible, auquel cas il est mis de côté (`accounts.json.invalide`) et les
/// sessions sur disque ne doivent pas être considérées comme orphelines.
fn load_store(dir: &Path) -> (Store, bool) {
    let path = dir.join("accounts.json");
    let (mut store, trusted) = match fs::read_to_string(&path) {
        Err(_) => (Store::default(), true),
        Ok(s) => match serde_json::from_str(&s) {
            Ok(store) => (store, true),
            Err(_) => {
                let _ = fs::rename(&path, dir.join("accounts.json.invalide"));
                (Store::default(), false)
            }
        },
    };
    if store.accounts.is_empty() {
        store.accounts = vec![Account::new(new_id(), "Compte 1".into())];
        store.active = Some(store.accounts[0].id.clone());
    }
    (store, trusted)
}

fn new_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("a{ms}")
}

fn service_url(service: &str) -> Option<&'static str> {
    match service {
        "gmail" => Some(GMAIL),
        "calendar" => Some("https://calendar.google.com/calendar/u/0/r"),
        "drive" => Some("https://drive.google.com/drive/u/0/"),
        "meet" => Some("https://meet.google.com/landing?authuser=0"),
        _ => None,
    }
}

/// Place la barre latérale et la vue du compte actif selon la taille de la fenêtre.
fn layout(app: &AppHandle) -> Result<(), String> {
    let Some(win) = app.get_window("main") else {
        return Ok(());
    };
    let scale = win.scale_factor().map_err(|e| e.to_string())?;
    let size = win
        .inner_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    for (label, wv) in app.webviews() {
        if label == "sidebar" {
            wv.set_position(LogicalPosition::new(0.0, 0.0))
                .map_err(|e| e.to_string())?;
            // Toute la fenêtre : les vues de comptes la recouvrent, sauf pendant un chargement,
            // où son loader apparaît à leur place.
            wv.set_size(LogicalSize::new(size.width, size.height))
                .map_err(|e| e.to_string())?;
        } else if label.starts_with("acct-") {
            wv.set_position(LogicalPosition::new(SIDEBAR_W, 0.0))
                .map_err(|e| e.to_string())?;
            wv.set_size(LogicalSize::new((size.width - SIDEBAR_W).max(0.0), size.height))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn set_loading(app: &AppHandle, on: bool) {
    if let Some(sidebar) = app.get_webview("sidebar") {
        let _ = sidebar.eval(format!("setLoading({on})"));
    }
}

/// Affiche la vue d'un compte une fois sa page chargée, et arrête le loader.
fn reveal(app: &AppHandle, label: &str) {
    if let Some(wv) = app.get_webview(label) {
        let _ = wv.show();
        set_loading(app, false);
    }
}

/// Traite un lien qui demande un nouvel onglet (`target="_blank"`, `window.open`), ce qu'une vue
/// de compte ne sait pas ouvrir : un service Google s'ouvre dans la vue elle-même, pour rester
/// dans la session du compte ; tout autre site s'ouvre dans le navigateur par défaut.
fn open_link(app: &AppHandle, label: &str, url: Url) {
    if !matches!(url.scheme(), "http" | "https") {
        return;
    }
    let host = url.host_str().unwrap_or_default();
    let google = host == "google.com" || host.ends_with(".google.com");
    // `google.com/url` est la redirection de Gmail vers un lien externe contenu dans un message
    let redirect = host == "www.google.com" && url.path() == "/url";
    if google && !redirect {
        if let Some(wv) = app.get_webview(label) {
            let _ = wv.navigate(url);
        }
    } else {
        let _ = std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url.as_str()])
            .spawn();
    }
}

/// Affiche le compte actif : ferme toutes les autres vues de comptes, puis crée (ou réutilise)
/// la sienne. Un compte inactif n'a donc aucun processus web en mémoire ; sa session reste sur disque.
///
/// Le compte actif est relu sous le verrou `views` : si deux changements de compte se croisent,
/// tous deux affichent le dernier choisi, et une seule vue existe à la fois.
fn show_active(app: &AppHandle, url: Option<&str>) -> Result<(), String> {
    let state = app.state::<AppState>();
    let _views = state.views.lock().unwrap();
    let active = state.store.lock().unwrap().active.clone();
    let label = active.as_ref().map(|id| format!("acct-{id}"));
    for (l, wv) in app.webviews() {
        if l.starts_with("acct-") && Some(&l) != label.as_ref() {
            let _ = wv.close();
        }
    }
    let (Some(id), Some(label)) = (active, label) else {
        // Plus aucun compte : rien à charger
        set_loading(app, false);
        return Ok(());
    };
    if let Some(wv) = app.get_webview(&label) {
        if let Some(u) = url {
            let parsed: Url = u.parse().map_err(|_| "URL invalide".to_string())?;
            wv.navigate(parsed).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    // La vue reste masquée jusqu'à la fin du chargement : on voit le loader de la barre
    // latérale au lieu d'un rectangle blanc. Il ne doit pas tourner indéfiniment si la création échoue.
    set_loading(app, true);
    create_account_view(app, &state, &id, &label, url).inspect_err(|_| set_loading(app, false))
}

fn create_account_view(
    app: &AppHandle,
    state: &AppState,
    id: &str,
    label: &str,
    url: Option<&str>,
) -> Result<(), String> {
    let win = app.get_window("main").ok_or("fenêtre introuvable")?;
    let profile = state.dir.join("profiles").join(id);
    fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
    let target: Url = url
        .unwrap_or(GMAIL)
        .parse()
        .map_err(|_| "URL invalide".to_string())?;
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let loaded = Arc::new(AtomicBool::new(false));

    // data_directory : un dossier de cookies/stockage distinct par compte (sessions isolées)
    let builder = WebviewBuilder::new(label, WebviewUrl::External(target))
        .data_directory(profile)
        .background_color(CONTENT_BG)
        .initialization_script(PROFILE_SCRIPT)
        .on_new_window({
            let (app, label) = (app.clone(), label.to_string());
            move |url, _| {
                open_link(&app, &label, url);
                NewWindowResponse::Deny
            }
        })
        .on_page_load({
            let loaded = loaded.clone();
            move |wv, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    loaded.store(true, Ordering::SeqCst);
                    reveal(wv.app_handle(), wv.label());
                }
            }
        });
    let wv = win
        .add_child(
            builder,
            LogicalPosition::new(SIDEBAR_W, 0.0),
            LogicalSize::new(100.0, 100.0),
        )
        .map_err(|e| e.to_string())?;
    let _ = wv.hide();
    // Le chargement a pu se terminer avant `hide()` : la vue serait alors restée masquée
    if loaded.load(Ordering::SeqCst) {
        reveal(app, label);
    }
    // Filet de sécurité : si la fin de chargement n'est jamais signalée, on affiche quand même
    // la vue, à condition qu'elle n'ait pas été remplacée entre-temps.
    let (handle, label) = (app.clone(), label.to_string());
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(8));
        if handle.state::<AppState>().generation.load(Ordering::SeqCst) == generation {
            reveal(&handle, &label);
        }
    });
    layout(app)
}

#[derive(Serialize)]
struct View {
    accounts: Vec<Account>,
    active: Option<String>,
    light: bool,
}

fn view(store: &Store) -> View {
    View {
        accounts: store.accounts.clone(),
        active: store.active.clone(),
        light: store.light,
    }
}

/// Bascule entre thème clair et sombre : barre latérale (via l'état renvoyé) et barres de titre.
#[tauri::command]
fn set_theme(app: AppHandle, state: State<AppState>, light: bool) -> View {
    let result = {
        let mut store = state.store.lock().unwrap();
        store.light = light;
        state.save(&store);
        view(&store)
    };
    for win in app.windows().values() {
        let _ = win.set_theme(window_theme(light));
    }
    result
}

#[tauri::command]
fn get_state(state: State<AppState>) -> View {
    view(&state.store.lock().unwrap())
}

// Les commandes qui créent une webview sont en `async` (hors du thread principal) :
// sous Windows, créer une webview depuis une commande synchrone bloque l'application (WebView2).
// Le verrou du store est relâché avant de toucher aux webviews.
#[tauri::command(async)]
fn switch_account(app: AppHandle, state: State<AppState>, id: String) -> Result<View, String> {
    let result = {
        let mut store = state.store.lock().unwrap();
        if !store.accounts.iter().any(|a| a.id == id) {
            return Err("compte inconnu".into());
        }
        store.active = Some(id);
        state.save(&store);
        view(&store)
    };
    show_active(&app, None)?;
    Ok(result)
}

fn add_account(app: &AppHandle, state: &AppState, name: &str) -> Result<(), String> {
    let id = new_id();
    {
        let mut store = state.store.lock().unwrap();
        let name = if name.is_empty() {
            format!("Compte {}", store.accounts.len() + 1)
        } else {
            name.to_string()
        };
        store.accounts.push(Account::new(id.clone(), name));
        store.active = Some(id);
        state.save(&store);
    }
    show_active(app, None)
}

fn rename_account(state: &AppState, id: &str, name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("nom vide".into());
    }
    let mut store = state.store.lock().unwrap();
    if let Some(a) = store.accounts.iter_mut().find(|a| a.id == id) {
        a.name = name.to_string();
    }
    state.save(&store);
    Ok(())
}

fn check_color(color: &str) -> Result<(), String> {
    let valid =
        color.len() == 7 && color.starts_with('#') && color[1..].chars().all(|c| c.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err("couleur invalide".into())
    }
}

fn set_color(state: &AppState, id: &str, color: &str) -> Result<(), String> {
    check_color(color)?;
    let mut store = state.store.lock().unwrap();
    if let Some(a) = store.accounts.iter_mut().find(|a| a.id == id) {
        a.color = Some(color.to_lowercase());
    }
    state.save(&store);
    Ok(())
}

/// Taille (en pixels logiques) de la fenêtre de saisie du nom d'un compte
const PROMPT_SIZE: (f64, f64) = (360.0, 150.0);

/// Ouvre la fenêtre de saisie : nom d'un nouveau compte si `id` est absent, sinon renommage,
/// ou choix de la couleur de fond si `kind` vaut "color".
/// C'est une fenêtre à part car la barre latérale est trop étroite pour un champ de saisie.
#[tauri::command(async)]
fn open_prompt(
    app: AppHandle,
    state: State<AppState>,
    id: Option<String>,
    kind: Option<String>,
) -> Result<(), String> {
    for (label, win) in app.webview_windows() {
        if label.starts_with("prompt-") {
            let _ = win.close();
        }
    }
    let color_mode = kind.as_deref() == Some("color");
    if color_mode && id.is_none() {
        return Err("compte manquant".into());
    }
    let (name, color) = match &id {
        Some(id) => {
            let store = state.store.lock().unwrap();
            let account = store
                .accounts
                .iter()
                .find(|a| &a.id == id)
                .ok_or("compte inconnu")?;
            (account.name.clone(), account.color.clone())
        }
        None => (String::new(), None),
    };
    let title = match (&id, color_mode) {
        (_, true) => "Couleur de fond",
        (Some(_), false) => "Renommer le compte",
        (None, false) => "Nouveau compte",
    };

    let main = app.get_window("main").ok_or("fenêtre introuvable")?;
    let scale = main.scale_factor().map_err(|e| e.to_string())?;
    let pos = main
        .outer_position()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let size = main
        .outer_size()
        .map_err(|e| e.to_string())?
        .to_logical::<f64>(scale);
    let (w, h) = PROMPT_SIZE;

    let light = state.store.lock().unwrap().light;
    let config = serde_json::json!({ "id": id, "name": name, "color": color, "kind": kind, "light": light });
    let builder = WebviewWindowBuilder::new(
        &app,
        new_id().replacen('a', "prompt-", 1),
        WebviewUrl::App("prompt.html".into()),
    )
    .title(title)
    .theme(window_theme(light))
    .inner_size(w, h)
    .position(pos.x + (size.width - w) / 2.0, pos.y + (size.height - h) / 2.0)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .skip_taskbar(true)
    .initialization_script(format!("window.__PROMPT__ = {config};"));
    // Fenêtre « possédée » par la principale : elle reste au-dessus et se ferme avec elle.
    // Ne pas utiliser `parent_raw` : il crée une fenêtre enfant, masquée par les vues de comptes.
    #[cfg(windows)]
    let builder = builder.owner_raw(main.hwnd().map_err(|e| e.to_string())?);
    let win = builder.build().map_err(|e| e.to_string())?;
    // Quelle que soit la façon de fermer (validation, Annuler, Échap, croix), la barre réaffiche
    // l'état enregistré, ce qui efface un éventuel aperçu de couleur non validé.
    win.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            if let Some(sidebar) = app.get_webview("sidebar") {
                let _ = sidebar.eval("run('get_state')");
            }
        }
    });
    Ok(())
}

/// Valide la fenêtre de saisie : change la couleur du compte si `color` est fourni, sinon crée
/// ou renomme le compte, puis ferme la fenêtre.
#[tauri::command(async)]
fn submit_prompt(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<AppState>,
    id: Option<String>,
    name: Option<String>,
    color: Option<String>,
) -> Result<(), String> {
    let name = name.as_deref().unwrap_or_default().trim();
    match (&id, &color) {
        (Some(id), Some(color)) => set_color(&state, id, color)?,
        (None, Some(_)) => return Err("compte manquant".into()),
        (Some(id), None) => rename_account(&state, id, name)?,
        (None, None) => add_account(&app, &state, name)?,
    }
    // La fermeture réaffiche la barre latérale (voir `open_prompt`)
    let _ = webview.window().close();
    Ok(())
}

#[tauri::command]
fn close_prompt(webview: tauri::Webview) {
    let _ = webview.window().close();
}

/// Aperçu dans la barre latérale de la couleur choisie, sans l'enregistrer : la fermeture de la
/// fenêtre de saisie réaffiche l'état enregistré (voir `open_prompt`).
#[tauri::command]
fn preview_color(app: AppHandle, id: String, color: String) -> Result<(), String> {
    check_color(&color)?;
    if let Some(sidebar) = app.get_webview("sidebar") {
        let args = serde_json::json!([id, color]);
        let _ = sidebar.eval(format!("previewColor(...{args})"));
    }
    Ok(())
}

#[tauri::command(async)]
fn remove_account(app: AppHandle, state: State<AppState>, id: String) -> Result<View, String> {
    // Vérifié avant tout : l'identifiant sert à construire le chemin du dossier supprimé plus bas
    let (result, was_active) = {
        let mut store = state.store.lock().unwrap();
        if !store.accounts.iter().any(|a| a.id == id) {
            return Err("compte inconnu".into());
        }
        store.accounts.retain(|a| a.id != id);
        let was_active = store.active.as_deref() == Some(id.as_str());
        if was_active {
            store.active = store.accounts.first().map(|a| a.id.clone());
        }
        state.save(&store);
        (view(&store), was_active)
    };
    if was_active {
        // Ferme la vue du compte supprimé et affiche le suivant, s'il en reste un
        show_active(&app, None)?;
    } else if let Some(wv) = app.get_webview(&format!("acct-{id}")) {
        let _ = wv.close();
    }
    // Supprime la session de ce compte (cookies). Échoue si le moteur la tient encore :
    // le dossier sera alors supprimé au prochain démarrage (voir `remove_orphan_profiles`).
    let _ = fs::remove_dir_all(state.dir.join("profiles").join(&id));
    Ok(result)
}

/// Applique l'ordre des comptes choisi par glisser-déposer dans la barre latérale.
#[tauri::command]
fn reorder_accounts(state: State<AppState>, ids: Vec<String>) -> Result<View, String> {
    let mut store = state.store.lock().unwrap();
    let mut rest = std::mem::take(&mut store.accounts);
    let mut ordered = Vec::with_capacity(rest.len());
    for id in &ids {
        if let Some(i) = rest.iter().position(|a| &a.id == id) {
            ordered.push(rest.remove(i));
        }
    }
    // Un compte absent de la liste reçue (ajouté entre-temps) reste à la fin
    ordered.append(&mut rest);
    store.accounts = ordered;
    state.save(&store);
    Ok(view(&store))
}

/// Photo transmise par la vue du compte : image en data: URL, de taille bornée
fn is_avatar_data(url: &str) -> bool {
    let Some((mime, b64)) = url.strip_prefix("data:").and_then(|d| d.split_once(";base64,")) else {
        return false;
    };
    ["image/png", "image/jpeg", "image/webp", "image/gif"].contains(&mime)
        && b64.len() <= 100_000
        && b64
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+/=".contains(c))
}

/// Seule commande ouverte aux pages Google (voir `capabilities/accounts.json`).
/// Le compte concerné est déduit de la vue appelante, jamais des arguments.
#[tauri::command]
fn report_profile(
    app: AppHandle,
    webview: tauri::Webview,
    state: State<AppState>,
    email: String,
    avatar: Option<String>,
) -> Result<(), String> {
    let id = webview.label().strip_prefix("acct-").ok_or("vue inconnue")?;
    let email = email.trim().to_lowercase();
    let valid = email.len() <= 254
        && email.contains('@')
        && email
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "@.+-_".contains(c));
    if !valid {
        return Err("adresse invalide".into());
    }
    let avatar = avatar.filter(|u| is_avatar_data(u));
    {
        let mut store = state.store.lock().unwrap();
        let Some(a) = store.accounts.iter_mut().find(|a| a.id == id) else {
            return Err("compte inconnu".into());
        };
        let same_email = a.email.as_deref() == Some(email.as_str());
        // L'avatar peut apparaître après l'adresse : on ne l'efface que si le compte a changé.
        // Une ancienne adresse distante (avant le stockage en data: URL) n'est pas conservée.
        let kept = a.avatar.clone().filter(|u| is_avatar_data(u));
        let avatar = if same_email { avatar.or(kept) } else { avatar };
        if same_email && a.avatar == avatar {
            return Ok(());
        }
        a.email = Some(email);
        a.avatar = avatar;
        state.save(&store);
    }
    if let Some(sidebar) = app.get_webview("sidebar") {
        let _ = sidebar.eval("run('get_state')");
    }
    Ok(())
}

/// Menu contextuel natif d'un compte : la barre latérale est trop étroite pour un menu HTML,
/// qui serait masqué par la vue du compte. Le choix revient à la barre via `onAccountMenu`.
#[tauri::command(async)]
fn account_menu(app: AppHandle, id: String) -> Result<(), String> {
    let win = app.get_window("main").ok_or("fenêtre introuvable")?;
    let menu = MenuBuilder::new(&app)
        .text(format!("rename:{id}"), "Renommer")
        .text(format!("color:{id}"), "Couleur de fond")
        .text(format!("remove:{id}"), "Supprimer")
        .build()
        .map_err(|e| e.to_string())?;
    menu.popup(win).map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn open_service(app: AppHandle, state: State<AppState>, service: String) -> Result<(), String> {
    let url = service_url(&service).ok_or("service inconnu")?;
    if state.store.lock().unwrap().active.is_none() {
        return Ok(());
    }
    show_active(&app, Some(url))
}

/// Supprime les sessions des comptes qui n'existent plus. Au démarrage, aucune vue ne tient
/// leurs fichiers, contrairement au moment de la suppression du compte.
fn remove_orphan_profiles(dir: &Path, store: &Store) {
    let Ok(entries) = fs::read_dir(dir.join("profiles")) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !store.accounts.iter().any(|a| name == a.id.as_str()) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_state,
            set_theme,
            switch_account,
            open_prompt,
            submit_prompt,
            close_prompt,
            preview_color,
            remove_account,
            reorder_accounts,
            account_menu,
            open_service,
            report_profile
        ])
        .on_menu_event(|app, event| {
            let Some((action, id)) = event.id().as_ref().split_once(':') else {
                return;
            };
            if let Some(sidebar) = app.get_webview("sidebar") {
                let args = serde_json::json!([action, id]);
                let _ = sidebar.eval(format!("onAccountMenu(...{args})"));
            }
        })
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            fs::create_dir_all(&dir)?;
            let (store, trusted) = load_store(&dir);
            if trusted {
                remove_orphan_profiles(&dir, &store);
            }
            let (active, light) = (store.active.is_some(), store.light);
            let state = AppState {
                store: Mutex::new(store),
                dir,
                views: Mutex::new(()),
                generation: AtomicU64::new(0),
            };
            state.save(&state.store.lock().unwrap());
            app.manage(state);

            let window = WindowBuilder::new(app, "main")
                .background_color(BG)
                .title("MultiMail")
                .theme(window_theme(light))
                .inner_size(1280.0, 820.0)
                .min_inner_size(900.0, 600.0)
                .build()?;

            window.add_child(
                WebviewBuilder::new("sidebar", WebviewUrl::App("index.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(SIDEBAR_W, 820.0),
            )?;

            let handle = app.handle().clone();
            window.on_window_event(move |event| {
                if matches!(event, WindowEvent::Resized(_)) {
                    let _ = layout(&handle);
                }
            });

            if active {
                show_active(app.handle(), None)?;
            }
            layout(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application");
}
