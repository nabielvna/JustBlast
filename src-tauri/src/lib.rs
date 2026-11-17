pub mod api;
mod application;
pub mod device;
pub mod preferences;

#[cfg(feature = "desktop")]
pub fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .setup(|app| {
            let state = api::NativeState::with_store(
                preferences::Store::at(app.path().app_data_dir()?)
                    .map_err(std::io::Error::other)?,
            )
            .map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::commands::workspace_snapshot,
            api::commands::replace_workspace_board,
            api::commands::workspace_action,
            api::commands::explorer_page,
            api::commands::preview_candidate,
            api::commands::preferences_snapshot,
            api::commands::save_preferences,
            api::commands::history_snapshot,
            api::commands::device_request,
            api::commands::calibration_request,
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build JustBlast")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
                app.state::<api::NativeState>().shutdown();
            }
        });
}
