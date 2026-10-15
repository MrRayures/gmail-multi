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

## Icônes : Lucide uniquement

- Toute icône vient de [Lucide](https://lucide.dev) (licence ISC, texte dans `licenses/lucide.txt`). Pas d'autre bibliothèque, pas de logo de marque (Google ou autre), pas de dessin maison.
- Copier le SVG de l'icône (paquet `lucide-static`, dossier `icons/`) **dans la page**, pas en `<img>` : seul un SVG intégré prend la couleur du texte, donc du thème. Ne garder que `viewBox` et la classe `icon`, qui porte les attributs communs (`stroke: currentColor`, épaisseur 2, extrémités arrondies) : `<svg class="icon" viewBox="0 0 24 24">…</svg>`.
- L'icône de l'application (`src-tauri/icons/icon.svg`) est l'icône `mail` de Lucide sur fond Dracula. Après modification, régénérer les images avec `pnpm tauri icon src-tauri/icons/icon.svg -o <dossier temporaire>` et ne copier que `32x32.png`, `128x128.png`, `icon.png` et `icon.ico` (les seules listées dans `tauri.conf.json`).
