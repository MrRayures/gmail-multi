# MultiMail

Cette application existe pour répondre à un besoin personnel. Elle me permet d'utiliser plusieurs comptes Gmail et les services Google associés dans une même application tout en ayant une consommation mémoire minimale.
Si elle peut aussi servir à quelqu'un d'autre tant mieux.

🤖 Ce projet est codé avec l'aide de Claude AI. La suite est gérée et générée par lui-même avec mon approbation.

> Projet indépendant, non affilié à Google ni approuvé par Google. Gmail, Google Agenda, Google Drive et Google Meet sont des marques de Google LLC.

## Fonctionnement

- **Un seul compte chargé à la fois.** Quand vous changez de compte, la vue précédente est détruite : plus aucun processus web pour elle. Sa session (cookies) reste sur le disque, vous restez connecté.
- **Une session isolée par compte** : chaque compte a son propre dossier de données, donc pas de mélange `/u/0`, `/u/1`.
- **Raccourcis** Mail, Agenda, Drive et Meet dans la barre latérale : ils ouvrent le service pour le compte actif.
- **Avatars** : la photo du compte Google s'affiche dans la barre ; à défaut, une pastille avec l'initiale.
- **Clic droit sur un compte** : renommer, choisir la couleur de fond, supprimer. Le bouton **+** ajoute un compte ; un glisser-déposer change l'ordre.
- **Thème** sombre ou clair, sous le bouton **+**.

## Confidentialité

- La connexion se fait directement sur les pages de Google : l'application ne voit ni ne stocke aucun mot de passe.
- Aucun serveur, aucune télémétrie : rien ne quitte votre ordinateur, à part votre navigation vers Google.
- Les sessions (cookies, chiffrés par WebView2) et la liste des comptes (`accounts.json` : nom, adresse, avatar, couleur) sont dans le dossier de données de l'application, `%APPDATA%\fr.perso.multimail`. Supprimer un compte supprime sa session.

## Installation

Téléchargez l'installeur dans les [releases](../../releases). Il n'est pas signé : Windows SmartScreen peut afficher un avertissement « éditeur inconnu » (« Informations complémentaires », puis « Exécuter quand même »).

Windows 10 ou 11 avec WebView2 (déjà présent sur un système à jour).

## Développement

À installer une seule fois :

1. **Node.js** (LTS) : https://nodejs.org, puis **pnpm** : `npm install -g pnpm`
2. **Rust** : https://rustup.rs (accepter les options par défaut)
3. **Outils de compilation C++** : « Visual Studio Build Tools » avec la charge de travail « Développement Desktop en C++ ».

```
pnpm install
pnpm dev      # lance l'application en développement
pnpm build    # installeur dans src-tauri/target/release/bundle/nsis/
```

La première compilation est longue (plusieurs minutes), les suivantes sont rapides.

## Licence

[GPL-3.0-or-later](LICENSE) : vous pouvez utiliser, modifier et redistribuer ce logiciel, à condition que toute version redistribuée reste sous la même licence, avec son code source.

Les icônes viennent de [Lucide](https://lucide.dev), sous [licence ISC](licenses/lucide.txt).
