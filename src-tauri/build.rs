fn main() {
    #[cfg(feature = "desktop")]
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "workspace_snapshot",
            "replace_workspace_board",
            "workspace_action",
            "explorer_page",
            "preview_candidate",
            "preferences_snapshot",
            "save_preferences",
            "history_snapshot",
            "device_request",
            "calibration_request",
        ]),
    ))
    .expect("Failed to prepare JustBlast desktop assets");
}
