fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "storage_health",
        "catalog_list",
        "catalog_preferences",
        "catalog_taxonomy",
        "series_save",
        "series_favorite",
        "series_delete",
        "content_list",
        "content_save",
        "content_delete",
        "content_reorder",
        "watch_update",
        "watch_history",
        "library_statistics",
        "continue_watching",
        "backup_access",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("No se pudo preparar la aplicación MyList");
}
