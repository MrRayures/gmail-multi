# À faire pour la v0.1.2

Cocher une tâche une fois faite ; supprimer ce fichier une fois la v0.1.2 publiée.

- [x] **Améliorer le chargement au changement de compte** : la vue précédente est fermée et Gmail rechargé entièrement (c'est ce qui économise la mémoire).
  - Écartée : garder les vues ouvertes (≈ 700 Mo par compte gardé, contraire à l'objectif du projet).
  - Mesuré : ≈ 0,5 s de démarrage de WebView2, puis ≈ 4 s (2 à 6 s) de chargement de Gmail.
  - Fait : la vue s'affiche dès le début du chargement (`PageLoadEvent::Started`) ; on voit l'écran de chargement de Gmail au lieu du loader. Durée réelle inchangée, flash blanc accepté en thème sombre.

- [x] **Foncer le fond de la zone de contenu** (à droite de la barre, visible pendant le chargement d'un compte) pour qu'elle se détache de la barre latérale : aujourd'hui `--bg-dark` (`#21222c`) contre `--bg` (`#282a36`), trop proches.
  - Piste : Dracula « Background Darker » `#191a21` pour le thème sombre, et un équivalent plus marqué pour le thème clair (Alucard).
  - Changer aussi `CONTENT_BG` dans `main.rs` (fond des vues de comptes avant leur affichage), sinon un flash de l'ancienne couleur apparaît.

- [ ] **Mettre à jour le README** avec l'icône et une capture de l'application (capture sans données personnelles visibles).
  - [x] Icône en tête du README.
  - [ ] Capture : à fournir (fichier `docs/capture.png`).

- [x] **Documenter la protection des données locales** dans la rubrique Confidentialité du README :
  - cookies de session chiffrés par WebView2 (clé protégée par Windows/DPAPI, liée à la session) ;
  - cache des pages, données hors ligne de Gmail et `accounts.json` non chiffrés, protégés seulement par les droits du profil Windows ;
  - recommander le chiffrement de l'appareil / BitLocker ; pas de chiffrement propre à l'application (la clé serait sur la même machine).
  - Ajouter l'astuce mémoire : désactiver Google Chat dans Gmail (Paramètres → Chat et Meet) économise ≈ 200 Mo par compte qui ne s'en sert pas.

- [ ] **Corriger le téléchargement des pièces jointes** : cliquer sur « Télécharger » dans un mail ne fait rien.
  - Bug prioritaire : c'est une fonction de base de Gmail.
  - Pistes à vérifier : le lien de téléchargement passe par `on_new_window` → `open_link`, qui le navigue dans la vue ; le téléchargement lui-même n'est peut-être jamais pris en charge. Tauri propose `WebviewBuilder::on_download` pour choisir le dossier (Téléchargements de Windows) et laisser WebView2 enregistrer le fichier.

- [ ] **Fun facts pendant le chargement ?** (idée à trancher)
  - Depuis l'affichage dès le début du chargement, le loader de la barre ne reste visible qu'≈ 0,5 s : un texte n'aurait pas le temps d'être lu. Pendant le reste du chargement, c'est l'écran de Gmail qui s'affiche, et la barre latérale ne peut pas s'afficher par-dessus.
  - À décider : abandonner, ou trouver un autre emplacement (par exemple au premier lancement).

- [ ] **Indicateur visuel du service affiché** (Mail, Agenda, Drive, Meet) dans la barre latérale.
  - Piste : le backend déduit le service de l'adresse de la page (`on_page_load`), y compris quand on navigue depuis la page elle-même, et l'indique à la barre (`eval`, comme `setLoading`) ; la barre met le bouton correspondant en évidence, comme le compte actif.
