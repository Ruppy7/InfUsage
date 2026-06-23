mod plugin_host;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            tray::create(app)?;
            let _ = plugin_host::run_demo_provider(&plugin_host::InfUsageHost);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
