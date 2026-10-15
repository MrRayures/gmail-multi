# Gmail Multi

Petite application de bureau (Tauri 2) pour utiliser plusieurs comptes Gmail et les services Google associés, avec une consommation mémoire minimale.

## Fonctionnement

- **Un seul compte chargé à la fois.** Quand vous changez de compte, la vue précédente est détruite : plus aucun processus web pour elle. Sa session (cookies) reste sur le disque, vous restez connecté.
- **Une session isolée par compte** : chaque compte a son propre dossier de données, donc pas de mélange `/u/0`, `/u/1`.
- **Raccourcis** Mail, Agenda, Drive et Meet dans la barre latérale : ils ouvrent le service pour le compte actif.
- **Clic droit sur un avatar** : renommer ou supprimer le compte. Le bouton **+** ajoute un compte.
- Les comptes sont enregistrés dans `accounts.json` (dossier de données de l'application).

## Installation (Windows 10/11)

À faire une seule fois :

1. **Node.js** (LTS) : https://nodejs.org, puis **pnpm** : `npm install -g pnpm`
2. **Rust** : https://rustup.rs (accepter les options par défaut)
3. **Outils de compilation C++** : installer « Visual Studio Build Tools » avec la charge de travail « Développement Desktop en C++ ».
4. **WebView2** : déjà présent sur Windows 10/11 à jour.

## Lancer en développement

```
pnpm install
pnpm dev
```

La première compilation est longue (plusieurs minutes), les suivantes sont rapides.

## Créer l'installeur

```
pnpm build
```

L'installeur `.exe` se trouve dans `src-tauri/target/release/bundle/nsis/`. L'exécutable seul est dans `src-tauri/target/release/`.

## Premier lancement

Deux comptes vides « Compte 1 » et « Compte 2 » sont créés. Cliquez sur un compte, connectez-vous à Google, puis renommez-le (clic droit).

## Points à vérifier au prototype

- La connexion Google dans la vue (Google bloque parfois les navigateurs embarqués ; ici le moteur est celui d'Edge, ce qui est favorable).
- L'isolation : connectez deux comptes différents et vérifiez qu'ils restent bien séparés après un redémarrage.
- Les multi-webviews de Tauri 2 sont encore marquées « unstable » : en cas de souci d'affichage au redimensionnement, le signaler.

## Pas encore dans ce prototype

- Pastilles de messages non lus sur les avatars.
- Notifications de bureau.
- Démarrage automatique avec Windows.
