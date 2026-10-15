// Couleurs de fond des comptes, partagées par la barre latérale et la fenêtre de choix.
// Accents Dracula : clairs, donc la lettre de la pastille est en couleur de fond (#282a36).
const COLORS = ["#bd93f9", "#ff79c6", "#8be9fd", "#50fa7b", "#ffb86c", "#ff5555", "#f1fa8c"];
// Noms des couleurs, dans le même ordre : infobulle et nom accessible des boutons de choix
const COLOR_NAMES = ["Violet", "Rose", "Cyan", "Vert", "Orange", "Rouge", "Jaune"];

// Couleur choisie, ou à défaut une couleur stable déduite de l'identifiant du compte
const colorOf = (a) => {
  if (a.color) return a.color;
  let h = 0;
  for (const c of a.id) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return COLORS[h % COLORS.length];
};
