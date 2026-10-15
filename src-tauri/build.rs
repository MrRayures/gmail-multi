fn main() {
    // Déclarer les commandes génère une permission par commande (`allow-…`), ce qui permet
    // d'en ouvrir une seule (`report_profile`) aux pages Google : voir `capabilities/`.
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_state",
            "set_theme",
            "switch_account",
            "open_prompt",
            "submit_prompt",
            "close_prompt",
            "preview_color",
            "remove_account",
            "reorder_accounts",
            "account_menu",
            "open_service",
            "report_profile",
        ]),
    ))
    .expect("échec de la génération Tauri");
}
