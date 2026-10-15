---
paths:
  - ".github/**"
---

# CI et release (GitHub Actions)

- `ci.yml` : à chaque push sur `main` / `develop` et à chaque pull request, sur `windows-latest` : `cargo fmt --check`, puis `cargo clippy --all-targets -- -D warnings`. Pas de Node.js : le frontend n'a pas d'étape de build et `cargo` n'en a pas besoin.
- `release.yml` : sur un tag `v*`, vérifie que le tag correspond à la version de `tauri.conf.json`, construit l'application avec `tauri-apps/tauri-action` (pnpm, installeur NSIS) et crée une release GitHub en brouillon.
- Garder les workflows courts : pas de matrice (Windows uniquement), cache Rust via `Swatinem/rust-cache`, versions d'actions figées sur une version majeure (`@v4`).
