use std::{env, fs, path::PathBuf};

use justblast_desktop::api::models::*;
use ts_rs::TS;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ts_rs::Config::default();
    let declarations = [
        HardwareProfile::decl(&config),
        CalibrationScreen::decl(&config),
        CalibrationTouch::decl(&config),
        CalibrationTiming::decl(&config),
        CalibrationAction::decl(&config),
        CalibrationRequest::decl(&config),
        CalibrationView::decl(&config),
        DeviceInfoView::decl(&config),
        VisualCycleView::decl(&config),
        SlotAnchor::decl(&config),
        CaptureView::decl(&config),
        DeviceView::decl(&config),
        DeviceAction::decl(&config),
        DeviceRequest::decl(&config),
        SavedShape::decl(&config),
        ScoreWeights::decl(&config),
        AutomationSettings::decl(&config),
        DeviceSettings::decl(&config),
        Preferences::decl(&config),
        HistoryEntry::decl(&config),
        PieceView::decl(&config),
        WeightsView::decl(&config),
        ComboView::decl(&config),
        MoveView::decl(&config),
        StepView::decl(&config),
        MetricsView::decl(&config),
        PlanView::decl(&config),
        JobStatus::decl(&config),
        JobView::decl(&config),
        WorkspaceAction::decl(&config),
        ExplorerRequest::decl(&config),
        CandidateView::decl(&config),
        ExplorerView::decl(&config),
        CandidateRequest::decl(&config),
        Capabilities::decl(&config),
        WorkspaceView::decl(&config),
        ReplaceBoardRequest::decl(&config),
        ErrorCode::decl(&config),
        CommandError::decl(&config),
    ];
    let content = format!(
        "{}\n",
        declarations
            .map(|line| format!("export {line}"))
            .join("\n\n")
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui/src/generated/contracts.ts");
    if env::args().any(|arg| arg == "--check") {
        if fs::read_to_string(path)? != content {
            return Err("IPC types changed. Run bun run contracts:generate.".into());
        }
    } else {
        fs::create_dir_all(path.parent().ok_or("Missing output directory")?)?;
        fs::write(path, content)?;
    }
    Ok(())
}
