// Pas de console noire sous Windows en version release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{
    webview::WebviewBuilder, window::WindowBuilder, AppHandle, LogicalPosition, LogicalSize,
    Manager, State, Url, WebviewUrl, WindowEvent,
};

/// Largeur (en pixels logiques) de la barre latérale
const SIDEBAR_W: f64 = 68.0;
const GMAIL: &str = "https://mail.google.com/mail/u/0/";

#[derive(Serialize, Deserialize, Clone)]
struct Account {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Store {
    accounts: Vec<Account>,
    active: Option<String>,
}

struct AppState {
    store: Mutex<Store>,
    dir: PathBuf,
}

impl AppState {
    fn save(&self, store: &Store) {
        if let Ok(json) = serde_json::to_string_pretty(store) {
            let _ = fs::write(self.dir.join("accounts.json"), json);
        }
    }
}

fn load_store(dir: &PathBuf) -> Store {
    let mut store: Store = fs::read_to_string(dir.join("accounts.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if store.accounts.is_empty() {
        store.accounts = vec![
            Account { id: new_id(0), name: "Compte 1".into() },
            Account { id: new_id(1), name: "Compte 2".into() },
        ];
        store.active = Some(store.accounts[0].id.clone());
    }
    store
}

fn new_id(salt: u128) -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("a{}", ms + salt)
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
            wv.set_position(LogicalPosition::new(0.0, 0.0)).map_err(|e| e.to_string())?;
            wv.set_size(LogicalSize::new(SIDEBAR_W, size.height)).map_err(|e| e.to_string())?;
        } else if label.starts_with("acct-") {
            wv.set_position(LogicalPosition::new(SIDEBAR_W, 0.0)).map_err(|e| e.to_string())?;
            wv.set_size(LogicalSize::new((size.width - SIDEBAR_W).max(0.0), size.height))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Ferme toutes les vues de comptes sauf celle demandée, puis crée (ou réutilise) la vue de ce compte.
/// Un compte inactif n'a donc aucun processus web en mémoire ; sa session reste sur disque.
fn show_account(app: &AppHandle, id: &str, url: Option<&str>) -> Result<(), String> {
    let label = format!("acct-{id}");
    for (l, wv) in app.webviews() {
        if l.starts_with("acct-") && l != label {
            let _ = wv.close();
        }
    }
    if let Some(wv) = app.get_webview(&label) {
        if let Some(u) = url {
            let parsed: Url = u.parse().map_err(|_| "URL invalide".to_string())?;
            wv.navigate(parsed).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let win = app.get_window("main").ok_or("fenêtre introuvable")?;
    let profile = app.state::<AppState>().dir.join("profiles").join(id);
    fs::create_dir_all(&profile).map_err(|e| e.to_string())?;
    let target: Url = url.unwrap_or(GMAIL).parse().map_err(|_| "URL invalide".to_string())?;

    // data_directory : un dossier de cookies/stockage distinct par compte (sessions isolées)
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(target)).data_directory(profile);
    win.add_child(
        builder,
        LogicalPosition::new(SIDEBAR_W, 0.0),
        LogicalSize::new(100.0, 100.0),
    )
    .map_err(|e| e.to_string())?;
    layout(app)
}

#[derive(Serialize)]
struct View {
    accounts: Vec<Account>,
    active: Option<String>,
}

fn view(store: &Store) -> View {
    View { accounts: store.accounts.clone(), active: store.active.clone() }
}

#[tauri::command]
fn get_state(state: State<AppState>) -> View {
    view(&state.store.lock().unwrap())
}

#[tauri::command]
fn switch_account(app: AppHandle, state: State<AppState>, id: String) -> Result<View, String> {
    let mut store = state.store.lock().unwrap();
    if !store.accounts.iter().any(|a| a.id == id) {
        return Err("compte inconnu".into());
    }
    store.active = Some(id.clone());
    state.save(&store);
    show_account(&app, &id, None)?;
    Ok(view(&store))
}

#[tauri::command]
fn add_account(app: AppHandle, state: State<AppState>, name: String) -> Result<View, String> {
    let mut store = state.store.lock().unwrap();
    let name = name.trim();
    let name = if name.is_empty() { format!("Compte {}", store.accounts.len() + 1) } else { name.to_string() };
    let id = new_id(0);
    store.accounts.push(Account { id: id.clone(), name });
    store.active = Some(id.clone());
    state.save(&store);
    show_account(&app, &id, None)?;
    Ok(view(&store))
}

#[tauri::command]
fn rename_account(state: State<AppState>, id: String, name: String) -> Result<View, String> {
    let mut store = state.store.lock().unwrap();
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("nom vide".into());
    }
    if let Some(a) = store.accounts.iter_mut().find(|a| a.id == id) {
        a.name = name;
    }
    state.save(&store);
    Ok(view(&store))
}

#[tauri::command]
fn remove_account(app: AppHandle, state: State<AppState>, id: String) -> Result<View, String> {
    let mut store = state.store.lock().unwrap();
    store.accounts.retain(|a| a.id != id);
    if let Some(wv) = app.get_webview(&format!("acct-{id}")) {
        let _ = wv.close();
    }
    // Supprime la session de ce compte (cookies). Peut échouer si le moteur la tient encore : sans gravité.
    let _ = fs::remove_dir_all(state.dir.join("profiles").join(&id));
    if store.active.as_deref() == Some(id.as_str()) {
        store.active = store.accounts.first().map(|a| a.id.clone());
        if let Some(next) = store.active.clone() {
            show_account(&app, &next, None)?;
        }
    }
    state.save(&store);
    Ok(view(&store))
}

#[tauri::command]
fn open_service(app: AppHandle, state: State<AppState>, service: String) -> Result<(), String> {
    let url = service_url(&service).ok_or("service inconnu")?;
    let active = state.store.lock().unwrap().active.clone();
    match active {
        Some(id) => show_account(&app, &id, Some(url)),
        None => Ok(()),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_state,
            switch_account,
            add_account,
            rename_account,
            remove_account,
            open_service
        ])
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            fs::create_dir_all(&dir)?;
            let store = load_store(&dir);
            let active = store.active.clone();
            let state = AppState { store: Mutex::new(store), dir };
            state.save(&state.store.lock().unwrap());
            app.manage(state);

            let window = WindowBuilder::new(app, "main")
                .title("Gmail Multi")
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

            if let Some(id) = active {
                show_account(app.handle(), &id, None)?;
            }
            layout(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application");
}
