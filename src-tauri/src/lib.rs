// ai-generated
mod classify;
mod commands;
mod downloads;
mod engine;
mod session;
mod shell;
mod store;
mod toy;
// ai-generated end

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            crate::shell::create_main_window(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::navigate,
            commands::new_tab,
            commands::close_tab,
            commands::switch_tab,
            commands::go_back,
            commands::go_forward,
            commands::reload,
            commands::toggle_bookmark,
            commands::list_bookmarks,
            commands::list_history,
            commands::search_history,
            commands::set_library_open,
            commands::set_engine,
            commands::list_downloads,
            commands::set_downloads_open,
            commands::open_download,
            commands::reveal_download,
            commands::clear_downloads,
            commands::set_find_open,
            commands::find_in_page
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
