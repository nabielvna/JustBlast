use std::io::{self, BufRead, Write};

use justblast_desktop::api::NativeState;
use serde_json::{Value, json};

#[path = "../../tests/support/mod.rs"]
mod phone_support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = if std::env::args().any(|a| a == "--fake-phone" || a == "--uncertain-phone") {
        let mut phone = phone_support::Phone::default();
        if std::env::args().any(|a| a == "--uncertain-phone") {
            phone.slots[0] = Some(phone_support::custom_ring());
        }
        NativeState::with_transport(
            match std::env::args().find_map(|a| a.strip_prefix("--store=").map(str::to_owned)) {
                Some(root) => justblast_desktop::preferences::Store::at(root.into())?,
                None => justblast_desktop::preferences::Store::default(),
            },
            Box::new(phone_support::Fake(std::sync::Arc::new(
                std::sync::Mutex::new(phone),
            ))),
        )?
    } else {
        NativeState::default()
    };
    let mut output = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let input: Value = serde_json::from_str(&line?)?;
        let args = input.get("args").cloned().unwrap_or_else(|| json!({}));
        let result = match input["command"].as_str() {
            Some("workspace_snapshot") => state
                .snapshot()
                .map(|v| serde_json::to_value(v).expect("Serialize workspace")),
            Some("replace_workspace_board") => state
                .replace_board(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize workspace")),
            Some("workspace_action") => state
                .action(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize workspace")),
            Some("explorer_page") => state
                .explorer_page(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize page")),
            Some("preview_candidate") => state
                .preview_candidate(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize plan")),
            Some("preferences_snapshot") => state
                .preferences()
                .map(|v| serde_json::to_value(v).expect("Serialize preferences")),
            Some("save_preferences") => state
                .save_preferences(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize preferences")),
            Some("history_snapshot") => state
                .history()
                .map(|v| serde_json::to_value(v).expect("Serialize history")),
            Some("calibration_request") => state
                .calibration_request(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize calibration")),
            Some("device_request") => state
                .device_request(serde_json::from_value(args["request"].clone())?)
                .map(|v| serde_json::to_value(v).expect("Serialize device")),
            _ => return Err("Unsupported test command".into()),
        };
        let response = match result {
            Ok(value) => json!({ "ok": true, "value": value }),
            Err(error) => json!({ "ok": false, "error": error }),
        };
        writeln!(output, "{response}")?;
        output.flush()?;
    }
    state.shutdown();
    Ok(())
}
