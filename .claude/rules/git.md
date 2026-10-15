# Git : branches, commits et versions

## Branches

- `main` : uniquement les versions publiées. Chaque fusion dans `main` est suivie d'un tag `vX.Y.Z`.
- `develop` : branche d'intégration, toujours compilable (`cargo clippy` sans avertissement).
- Une branche par sujet, créée depuis `develop` : `feat/<sujet>`, `fix/<sujet>`, `refactor/<sujet>`, `docs/<sujet>`, `ci/<sujet>`, `chore/<sujet>` (sujet court en kebab-case, par exemple `fix/ecriture-accounts-json`). Elle est fusionnée dans `develop` par une pull request GitHub, après passage de la CI.
- Ne jamais commiter directement sur `main`.

## Commits

- Message en français : `type: description` au présent, sans majuscule ni point final (`fix: écrit accounts.json de façon atomique`). Types : `feat`, `fix`, `refactor`, `docs`, `ci`, `chore`.
- Un commit = un changement cohérent ; le code compile à chaque commit.
- Avant de commiter du Rust : `cargo clippy --all-targets -- -D warnings` dans `src-tauri/`.
- Ne commiter, pousser, fusionner ou taguer qu'à la demande de l'utilisateur.

## Publier une version

1. Sur `develop`, mettre le même numéro `X.Y.Z` dans `package.json`, `src-tauri/tauri.conf.json` et `src-tauri/Cargo.toml`, puis `cargo check` (met à jour `Cargo.lock`). Commit `chore: version X.Y.Z`.
2. Fusionner `develop` dans `main` (pull request), puis taguer `main` : `git tag vX.Y.Z` et `git push origin vX.Y.Z`.
3. Le tag déclenche `.github/workflows/release.yml`, qui construit l'installeur NSIS et crée une release GitHub en **brouillon**, à vérifier puis publier à la main.

Le workflow de release échoue si le tag ne correspond pas à la version de `tauri.conf.json`.
