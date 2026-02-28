mod backup;
mod commands;
mod error;
mod models;
mod parsers;
mod services;
mod validators;
mod writers;

use tauri::Manager;

use services::vehicle_image::VehicleImageService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let cache_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir")
                .join("cache")
                .join("vehicle-images");
            let service =
                VehicleImageService::new(cache_dir).expect("failed to create image service");
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::savegame::greet,
            commands::savegame::list_savegames,
            commands::savegame::load_savegame,
            commands::savegame::save_changes,
            commands::backup::list_backups,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::delete_backup,
            commands::vehicle_image::detect_game_path,
            commands::vehicle_image::get_vehicle_images_batch,
            commands::vehicle_image::clear_image_cache,
            commands::vehicle_image::get_image_cache_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
