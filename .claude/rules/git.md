# Git : branches, commits et versions

## Branches

Projet mené seul : pas de pull request obligatoire.

- `develop` : on y commite et pousse directement. La CI tourne à chaque push ; la branche doit rester compilable (voir « Commits »).
- `main` : uniquement les versions publiées, reçues de `develop` au moment d'une release (voir « Publier une version »). Ne jamais commiter directement sur `main`.
- Branche temporaire facultative, créée depuis `develop`, pour un changement risqué ou long qu'on veut pouvoir abandonner : `feat/<sujet>`, `fix/<sujet>`… (sujet court en kebab-case). Elle est fusionnée en local dans `develop`, puis supprimée.

## Commits

- Message en français : `type: description` au présent, sans majuscule ni point final (`fix: écrit accounts.json de façon atomique`). Types : `feat`, `fix`, `refactor`, `docs`, `ci`, `chore`.
- Un commit = un changement cohérent ; le code compile à chaque commit.
- Avant de commiter du Rust : `cargo fmt --check` et `cargo clippy --all-targets -- -D warnings` dans `src-tauri/`, les mêmes vérifications que la CI.
- Ne commiter, pousser, fusionner ou taguer qu'à la demande de l'utilisateur.

## Publier une version

1. Sur `develop`, mettre le même numéro `X.Y.Z` dans `package.json`, `src-tauri/tauri.conf.json` et `src-tauri/Cargo.toml`, puis `cargo check` (met à jour `Cargo.lock`). Commit `chore: version X.Y.Z`.
2. Vérifier que la CI de `develop` est verte et que l'application a été testée à la main, puis fusionner `develop` dans `main` (`git checkout main && git merge --ff-only develop`, ou `git merge develop` si `main` a divergé) et pousser `main`. Taguer `main` : `git tag vX.Y.Z` et `git push origin vX.Y.Z`.
3. Le tag déclenche `.github/workflows/release.yml`, qui construit l'installeur NSIS et crée une release GitHub en **brouillon**, à vérifier puis publier à la main.

Le workflow de release échoue si le tag ne correspond pas à la version de `tauri.conf.json`.
