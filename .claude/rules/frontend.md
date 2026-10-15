---
paths:
  - "src/**"
  - "src-tauri/icons/**"
---

# Frontend (`src/`)

- HTML/CSS/JS sans framework, sans bundler ni dépendance : un fichier partagé est un simple `<script src>` (comme `colors.js`).
- Thèmes : Dracula (sombre, par défaut) et Alucard (clair, classe `light` sur `body` dans `index.html`, sur `html` dans `prompt.html`). Toute couleur d'interface passe par les variables CSS de `:root`, redéfinies pour le thème clair ; pas de couleur en dur, sauf les pastilles de comptes (palette `COLORS` de `colors.js`, identique dans les deux thèmes, lettre toujours en `#282a36`).
- Les fonctions appelées par le backend via `eval` (`run`, `onAccountMenu`, `setLoading`, `previewColor`) doivent rester globales ; les renommer impose de modifier `main.rs`.
- Le glisser-déposer HTML5 est intercepté par Tauri : la réorganisation des comptes utilise les événements pointeur. Pendant un déplacement, `run()` diffère le `render()` (`pendingRender`).
- Pas d'animation permanente : le loader ne tourne que pendant un chargement (`body.loading`).

## HTML

- `<!doctype html>`, `lang="fr"`, `charset` et `viewport` dans chaque page.
- Éléments natifs pour leur rôle : `<button type="button">` pour une action (focalisable et activable au clavier sans code), `<form>` + `submit` pour une saisie, `<label for>` pour un champ.
- Un bouton sans texte visible a un nom accessible (`title`, ou `aria-label` s'il n'a pas d'infobulle) ; une image décorative a `alt=""`, un SVG décoratif `aria-hidden="true"`.
- Pas de gestionnaire d'événement ni de style dans les attributs (`onclick=`, `style=`) : la CSP les bloque. Les événements se branchent en JS, les styles dynamiques passent par `el.style` ou une classe.

## CSS

- Couleurs uniquement par variables (voir Thèmes ci-dessus) ; pas de `!important` ; sélecteurs simples (id ou classe, peu d'imbrication).
- Ne jamais supprimer le contour de focus (`outline: none`) sans un remplacement visible (`:focus` / `:focus-visible`).
- Animations et transitions courtes, sur `opacity` et `transform` de préférence, et neutralisées par `@media (prefers-reduced-motion: reduce)`.
- Dimensions de la barre latérale en accord avec `SIDEBAR_W` (68 px) dans `main.rs`.

## JavaScript

- `const` par défaut, `let` si la variable change, jamais `var` ; égalité stricte (`===`, `!==`).
- Données affichées avec `textContent`, jamais `innerHTML` (noms, adresses : risque d'injection). `innerHTML = ""` pour vider un conteneur est admis.
- Tout `invoke` a un `.catch(console.error)` ou un `try/catch` : une promesse rejetée sans traitement est une erreur silencieuse.
- Pas de minuterie permanente (`setInterval`) ni d'écouteur ajouté à chaque `render()` sur un élément qui persiste.
- Pas de dépendance ni de module ES : scripts classiques, dans l'ordre (`colors.js` avant le script de la page).

## Icônes : Lucide uniquement

- Toute icône vient de [Lucide](https://lucide.dev) (licence ISC, texte dans `licenses/lucide.txt`). Pas d'autre bibliothèque, pas de logo de marque (Google ou autre), pas de dessin maison.
- Copier le SVG de l'icône (paquet `lucide-static`, dossier `icons/`) **dans la page**, pas en `<img>` : seul un SVG intégré prend la couleur du texte, donc du thème. Ne garder que `viewBox` et la classe `icon`, qui porte les attributs communs (`stroke: currentColor`, épaisseur 2, extrémités arrondies) : `<svg class="icon" viewBox="0 0 24 24">…</svg>`.
- L'icône de l'application (`src-tauri/icons/icon.svg`) est l'icône `mail` de Lucide sur fond Dracula. Après modification, régénérer les images avec `pnpm tauri icon src-tauri/icons/icon.svg -o <dossier temporaire>` et ne copier que `32x32.png`, `128x128.png`, `icon.png` et `icon.ico` (les seules listées dans `tauri.conf.json`).
