mod backup;
mod commands;
mod error;
mod models;
mod parsers;
mod services;
mod validators;
mod writers;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::savegame::greet,
            commands::savegame::list_savegames,
            commands::savegame::load_savegame,
            commands::savegame::save_changes,
            commands::backup::list_backups,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::delete_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
