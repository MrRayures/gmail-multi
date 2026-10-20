<img src="src-tauri/icons/128x128.png" alt="" width="64" height="64">

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
- Les données de l'application sont dans `%APPDATA%\fr.perso.multimail` : un dossier de session par compte, et la liste des comptes (`accounts.json` : nom, adresse, avatar, couleur). Supprimer un compte supprime sa session.

### Protection des données sur l'ordinateur

| Donnée | Protection |
|---|---|
| Cookies de session (ce qui vous garde connecté) | **Chiffrés** par WebView2, avec une clé protégée par Windows (DPAPI) et liée à votre session Windows |
| Cache des pages, données hors ligne de Gmail | Non chiffrés : protégés par les droits d'accès de votre profil Windows |
| `accounts.json` | Non chiffré : mêmes droits d'accès |

Un autre utilisateur de l'ordinateur n'a pas accès à ces fichiers, et vos cookies ne sont pas réutilisables sans votre mot de passe Windows. En revanche, le cache et `accounts.json` restent lisibles sur un disque non chiffré, et aucun navigateur ne protège contre un logiciel malveillant qui tourne sous votre compte.

Pour tout protéger, activez le **chiffrement de l'appareil** ou **BitLocker** (Paramètres Windows → Confidentialité et sécurité → Chiffrement de l'appareil). L'application n'ajoute pas son propre chiffrement : sa clé serait stockée sur la même machine, donc protégée de la même façon que les cookies.

## Astuce : réduire la mémoire

Dans Gmail, le panneau Google Chat intégré occupe à lui seul environ 200 Mo. Pour un compte qui n'utilise pas Chat, désactivez-le dans Gmail : **Paramètres → Voir tous les paramètres → Chat et Meet → Chat : Désactivé**. Le réglage est propre à chaque compte.

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
