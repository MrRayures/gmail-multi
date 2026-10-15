# À faire pour la v0.1.2

Cocher une tâche une fois faite ; supprimer ce fichier une fois la v0.1.2 publiée.

- [ ] **Changer l'icône de l'application** : le carré violet plein « fait tache » dans la barre des tâches, où les autres icônes sont des formes détourées sur fond transparent (style Windows 11).
  - Piste : enveloppe `mail` de Lucide en violet Dracula, sans fond, trait plus épais pour rester nette à 16 et 24 px.
  - Régénérer les images avec `pnpm tauri icon` (procédure dans `.claude/rules/frontend.md`).

- [ ] **Améliorer le chargement au changement de compte** : la vue précédente est fermée et Gmail rechargé entièrement (c'est ce qui économise la mémoire).
  - Écartée : garder les vues ouvertes (≈ 700 Mo par compte gardé, contraire à l'objectif du projet).
  - Piste : afficher la vue dès que Gmail montre son propre écran de chargement, au lieu d'attendre la fin complète du chargement (`reveal` sur `PageLoadEvent::Finished`).

- [ ] **Mettre à jour le README** avec l'icône et une capture de l'application (après le changement d'icône ; capture sans données personnelles visibles).

- [ ] **Documenter la protection des données locales** dans la rubrique Confidentialité du README :
  - cookies de session chiffrés par WebView2 (clé protégée par Windows/DPAPI, liée à la session) ;
  - cache des pages, données hors ligne de Gmail et `accounts.json` non chiffrés, protégés seulement par les droits du profil Windows ;
  - recommander le chiffrement de l'appareil / BitLocker ; pas de chiffrement propre à l'application (la clé serait sur la même machine).
  - Ajouter l'astuce mémoire : désactiver Google Chat dans Gmail (Paramètres → Chat et Meet) économise ≈ 200 Mo par compte qui ne s'en sert pas.
