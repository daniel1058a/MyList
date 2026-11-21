mod commands;
mod domain;
mod infrastructure;
mod repositories;
mod services;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol("mylist-cover", |context, request, responder| {
            let location = context.app_handle().state::<commands::StorageLocation>();
            let root = location.root.clone();
            let gate = location.gate.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let image = if request.method() == tauri::http::Method::GET {
                    root.as_deref().and_then(|root| {
                        let _operation = gate.read().ok()?;
                        let key = request.uri().path().strip_prefix('/')?;
                        infrastructure::covers::thumbnail(root, key)
                    })
                } else {
                    None
                };
                let response = tauri::http::Response::builder()
                    .header("X-Content-Type-Options", "nosniff")
                    .header("Cache-Control", "no-store");
                let response = match image {
                    Some(bytes) => response.header("Content-Type", "image/webp").body(bytes),
                    None => response.status(404).body(Vec::new()),
                }
                .expect("Respuesta de portada válida");
                responder.respond(response);
            });
        })
        .setup(|app| {
            app.manage(commands::StorageLocation::new(
                app.path().app_data_dir().ok(),
            ));
            app.manage(commands::RestorePreview::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::storage_health,
            commands::catalog_list,
            commands::catalog_preferences,
            commands::catalog_taxonomy,
            commands::series_save,
            commands::series_favorite,
            commands::series_delete,
            commands::content_list,
            commands::content_save,
            commands::content_delete,
            commands::content_reorder,
            commands::watch_update,
            commands::watch_history,
            commands::library_statistics,
            commands::continue_watching,
            commands::backups::backup_access,
        ])
        .run(tauri::generate_context!())
        .expect("No se pudo iniciar MyList");
}
