---
paths:
  - "src/**"
---

# Frontend (`src/`)

- HTML/CSS/JS sans framework, sans bundler ni dépendance : un fichier partagé est un simple `<script src>` (comme `colors.js`).
- Thèmes : Dracula (sombre, par défaut) et Alucard (clair, classe `light` sur `body` dans `index.html`, sur `html` dans `prompt.html`). Toute couleur d'interface passe par les variables CSS de `:root`, redéfinies pour le thème clair ; pas de couleur en dur, sauf les pastilles de comptes (palette `COLORS` de `colors.js`, identique dans les deux thèmes, lettre toujours en `#282a36`).
- Les fonctions appelées par le backend via `eval` (`run`, `onAccountMenu`, `setLoading`, `previewColor`) doivent rester globales ; les renommer impose de modifier `main.rs`.
- Le glisser-déposer HTML5 est intercepté par Tauri : la réorganisation des comptes utilise les événements pointeur. Pendant un déplacement, `run()` diffère le `render()` (`pendingRender`).
- Pas d'animation permanente : le loader ne tourne que pendant un chargement (`body.loading`).
