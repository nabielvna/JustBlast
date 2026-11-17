use super::{NativeState, models::*};

async fn blocking<T: Send + 'static>(
    state: &NativeState,
    operation: impl FnOnce(NativeState) -> Result<T, CommandError> + Send + 'static,
) -> Result<T, CommandError> {
    let owned = state.clone();
    tauri::async_runtime::spawn_blocking(move || operation(owned))
        .await
        .map_err(|_| CommandError {
            code: ErrorCode::Internal,
            message: "A native operation failed. Refresh the workspace before trying again."
                .to_owned(),
        })?
}
#[tauri::command]
pub async fn workspace_snapshot(
    state: tauri::State<'_, NativeState>,
) -> Result<WorkspaceView, CommandError> {
    blocking(state.inner(), |state| state.snapshot()).await
}
#[tauri::command]
pub async fn replace_workspace_board(
    state: tauri::State<'_, NativeState>,
    request: ReplaceBoardRequest,
) -> Result<WorkspaceView, CommandError> {
    blocking(state.inner(), move |state| state.replace_board(request)).await
}
#[tauri::command]
pub async fn workspace_action(
    state: tauri::State<'_, NativeState>,
    request: WorkspaceAction,
) -> Result<WorkspaceView, CommandError> {
    blocking(state.inner(), move |state| state.action(request)).await
}
#[tauri::command]
pub async fn explorer_page(
    state: tauri::State<'_, NativeState>,
    request: ExplorerRequest,
) -> Result<ExplorerView, CommandError> {
    blocking(state.inner(), move |state| state.explorer_page(request)).await
}
#[tauri::command]
pub async fn preview_candidate(
    state: tauri::State<'_, NativeState>,
    request: CandidateRequest,
) -> Result<PlanView, CommandError> {
    blocking(state.inner(), move |state| state.preview_candidate(request)).await
}
#[tauri::command]
pub async fn preferences_snapshot(
    state: tauri::State<'_, NativeState>,
) -> Result<Preferences, CommandError> {
    blocking(state.inner(), |state| state.preferences()).await
}
#[tauri::command]
pub async fn save_preferences(
    state: tauri::State<'_, NativeState>,
    request: Preferences,
) -> Result<Preferences, CommandError> {
    blocking(state.inner(), move |state| state.save_preferences(request)).await
}
#[tauri::command]
pub async fn history_snapshot(
    state: tauri::State<'_, NativeState>,
) -> Result<Vec<HistoryEntry>, CommandError> {
    blocking(state.inner(), |state| state.history()).await
}
#[tauri::command]
pub async fn device_request(
    state: tauri::State<'_, NativeState>,
    request: DeviceRequest,
) -> Result<WorkspaceView, CommandError> {
    blocking(state.inner(), move |state| state.device_request(request)).await
}

#[tauri::command]
pub async fn calibration_request(
    state: tauri::State<'_, NativeState>,
    request: CalibrationRequest,
) -> Result<CalibrationView, CommandError> {
    blocking(state.inner(), move |state| {
        state.calibration_request(request)
    })
    .await
}
