# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Projet

Gmail Multi : application de bureau Tauri 2, Windows uniquement (WebView2, installeur NSIS), pour utiliser plusieurs comptes Gmail avec une consommation mémoire minimale. Le code, les commentaires et l'interface sont en français.

**Contrainte du projet : l'application doit rester performante et simple.** Si une demande risque de mettre cela en péril (webview ou processus supplémentaire, dépendance, animation ou interrogation permanente, étape de build), la questionner avant de l'implémenter et proposer une alternative plus légère.

Règles détaillées par thème dans `.claude/rules/` : `git.md` (branches, commits, versions), `backend.md` (Rust, chargée pour `src-tauri/`), `frontend.md` (chargée pour `src/`), `ci.md` (chargée pour `.github/`).

## Commandes

```
pnpm install
pnpm dev         # tauri dev : recompile le Rust et relance l'application à chaque modification
pnpm build       # installeur dans src-tauri/target/release/bundle/nsis/
```

Vérification du Rust sans lancer l'application, dans `src-tauri/` : `cargo check`, et `cargo clippy --all-targets -- -D warnings` (exigé par la CI). Les deux attendent le verrou de build si `tauri dev` tourne.

Il n'y a ni tests ni étape de build du frontend : `src/` est servi tel quel (`frontendDist: "../src"`), en HTML/CSS/JS sans framework ni bundler.

## Architecture

Tout le backend est dans `src-tauri/src/main.rs` ; le frontend tient en deux pages, `src/index.html` (barre latérale) et `src/prompt.html` (fenêtre de saisie du nom d'un compte ou de choix de sa couleur de fond), qui partagent la palette de `src/colors.js`.

### Une fenêtre, plusieurs webviews

La fenêtre `main` est créée en Rust (`windows: []` dans `tauri.conf.json`) et contient des webviews enfants, via la fonctionnalité `unstable` de Tauri (`Window::add_child`) :

- `sidebar` : `index.html`. Elle occupe **toute** la fenêtre, mais seule sa bande gauche (`SIDEBAR_W`, 68 px) est visible ; le reste est recouvert par la vue du compte.
- `acct-<id>` : la page Google du compte actif, placée à droite de la barre. **Une seule existe à la fois** (`show_active`). Chaque compte a son propre `data_directory` (`profiles/<id>` dans le dossier de données de l'application), donc des sessions isolées qui persistent sur disque.

Conséquence à garder en tête : **du HTML de la barre latérale ne peut jamais s'afficher par-dessus la vue d'un compte** (webview native distincte, au-dessus dans l'ordre d'empilement). Tout ce qui doit déborder de la barre passe par du natif : menu contextuel natif (`account_menu`), fenêtre séparée (`open_prompt`). À l'inverse, le loader de changement de compte est du HTML de la barre, visible uniquement parce que la vue du compte reste masquée jusqu'à la fin de son chargement (`reveal`).

### État et communication

- L'état (`Store` : comptes, compte actif, thème) est dans `AppState`, sauvegardé dans `accounts.json` à chaque modification.
- La barre latérale appelle les commandes avec `invoke` (`withGlobalTauri`) ; la plupart renvoient l'état complet (`View`), que `run()` réaffiche.
- Dans l'autre sens, le backend pilote la barre avec `eval` sur des fonctions globales de `index.html` (`run('get_state')`, `onAccountMenu`, `setLoading`, `previewColor`). Il n'y a pas d'événements Tauri, car ils demanderaient des permissions supplémentaires.
- Les vues de comptes reçoivent un script d'initialisation (`PROFILE_SCRIPT`) qui repère l'adresse et la photo du compte Google connecté et les signale par `report_profile`.
