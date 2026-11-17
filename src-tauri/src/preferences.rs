use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct SavedShape {
    pub id: String,
    pub name: String,
    pub cells: Vec<(u8, u8)>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct AutomationSettings {
    pub move_device: bool,
    pub continuous: bool,
    pub check_after_move: bool,
    pub check_after_cycle: bool,
    pub cycle_limit: u32,
}
impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            move_device: false,
            continuous: false,
            check_after_move: false,
            check_after_cycle: true,
            cycle_limit: 0,
        }
    }
}
fn initial_revision() -> u32 {
    1
}
fn default_profile() -> String {
    "default".to_owned()
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct DeviceSettings {
    pub serial: String,
    pub wireless_address: String,
    pub method: String,
    #[serde(default = "default_profile")]
    pub profile_id: String,
}
impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            serial: String::new(),
            wireless_address: String::new(),
            method: "usb".to_owned(),
            profile_id: default_profile(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct ScoreWeights {
    pub points: f64,
    pub combo: f64,
    pub flexibility: f64,
    pub fragmentation: f64,
}
impl Default for ScoreWeights {
    fn default() -> Self {
        justblast_core::solver::Weights::default().into()
    }
}
impl From<justblast_core::solver::Weights> for ScoreWeights {
    fn from(w: justblast_core::solver::Weights) -> Self {
        Self {
            points: w.points,
            combo: w.combo,
            flexibility: w.flexibility,
            fragmentation: w.fragmentation,
        }
    }
}
impl From<ScoreWeights> for justblast_core::solver::Weights {
    fn from(w: ScoreWeights) -> Self {
        Self {
            points: w.points,
            combo: w.combo,
            flexibility: w.flexibility,
            fragmentation: w.fragmentation,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Preferences {
    pub schema_version: u8,
    #[serde(default = "initial_revision")]
    pub revision: u32,
    pub shapes: Vec<SavedShape>,
    pub automation: AutomationSettings,
    pub connection: DeviceSettings,
    #[serde(default)]
    pub weights: ScoreWeights,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 2,
            revision: initial_revision(),
            shapes: Vec::new(),
            automation: AutomationSettings::default(),
            connection: DeviceSettings::default(),
            weights: ScoreWeights::default(),
        }
    }
}
impl Preferences {
    pub fn advance(&mut self) -> Result<(), String> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or("Preferences revision limit reached")?;
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if !justblast_core::solver::Weights::from(self.weights.clone()).valid()
            || self.revision == 0
            || self.schema_version != 2
            || self.shapes.len() > 500
            || self.automation.cycle_limit > 100_000
            || !["usb", "wireless"].contains(&self.connection.method.as_str())
            || !crate::device::profile::valid_id(&self.connection.profile_id)
            || self.connection.serial.len() > 256
            || self.connection.wireless_address.len() > 256
        {
            return Err("Invalid preferences".to_owned());
        }
        let mut ids = std::collections::HashSet::new();
        for shape in &self.shapes {
            if !shape.id.starts_with("custom-")
                || shape.id.len() > 100
                || !ids.insert(&shape.id)
                || shape.name.trim().is_empty()
                || shape.name.chars().count() > 40
                || shape.cells.len() > 25
                || shape.cells.iter().any(|&(r, c)| r > 4 || c > 4)
                || justblast_core::domain::Shape::new(shape.cells.clone()).is_err()
            {
                return Err("Invalid saved shape".to_owned());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct HistoryEntry {
    pub timestamp: String,
    pub source: String,
    pub cycle: u32,
    pub status: String,
    pub message: String,
    pub board: [String; 8],
}
#[derive(Clone, Default)]
pub struct Store {
    root: Option<PathBuf>,
}
impl Store {
    pub fn at(root: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        Ok(Self { root: Some(root) })
    }
    pub fn load_preferences(&self) -> Result<Preferences, String> {
        let Some(root) = &self.root else {
            return Ok(Preferences::default());
        };
        let path = root.join("preferences.json");
        if !path.exists() {
            return Ok(Preferences::default());
        }
        if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 1_000_000 {
            return Err("Preferences file is too large".to_owned());
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if bytes.len() > 1_000_000 {
            return Err("Preferences file is too large".to_owned());
        }
        let mut stored: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Could not read preferences: {e}"))?;
        let migrate_schema = stored.get("schemaVersion").and_then(|v| v.as_u64()) == Some(1);
        let rename_builtin = stored["connection"]["profileId"].as_str() == Some("samsung-a73");
        if rename_builtin {
            stored["connection"]["profileId"] = serde_json::json!(default_profile());
        }
        if migrate_schema {
            let original = root.join("preferences.v1.backup.json");
            if !original.exists() {
                atomic_write(&original, &bytes)?;
            }
            stored
                .as_object_mut()
                .ok_or("Invalid preferences")?
                .remove("mode");
            stored["schemaVersion"] = serde_json::json!(2);
        }
        if let Some(automation) = stored.get_mut("automation").and_then(|v| v.as_object_mut()) {
            automation.remove("animation");
            automation.remove("confirmCycle");
            automation.remove("confirmMove");
        }
        let prefs: Preferences = serde_json::from_value(stored)
            .map_err(|e| format!("Could not read preferences: {e}"))?;
        prefs.validate()?;
        if migrate_schema || rename_builtin {
            self.save_preferences(&prefs)?;
        }
        Ok(prefs)
    }
    pub fn save_preferences(&self, prefs: &Preferences) -> Result<(), String> {
        prefs.validate()?;
        if let Some(root) = &self.root {
            atomic_write(
                &root.join("preferences.json"),
                &serde_json::to_vec_pretty(prefs).map_err(|e| e.to_string())?,
            )?;
        }
        Ok(())
    }
    pub fn load_history(&self) -> Result<Vec<HistoryEntry>, String> {
        let Some(root) = &self.root else {
            return Ok(Vec::new());
        };
        let path = root.join("history.json");
        if !path.exists() {
            return Ok(Vec::new());
        }
        if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 2_000_000 {
            return Err("History file is too large".to_owned());
        }
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if bytes.len() > 2_000_000 {
            return Err("History file is too large".to_owned());
        }
        let mut stored: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("Could not read history: {e}"))?;
        if let Some(entries) = stored.as_array_mut() {
            for entry in entries {
                if let Some(object) = entry.as_object_mut()
                    && !object.contains_key("source")
                    && let Some(source) = object.remove("mode")
                {
                    object.insert("source".to_owned(), source);
                }
            }
        }
        let entries: Vec<HistoryEntry> =
            serde_json::from_value(stored).map_err(|e| format!("Could not read history: {e}"))?;
        if entries.len() > 200
            || entries.iter().any(|e| {
                e.cycle == 0
                    || !["manual", "auto", "phone", "simulated"].contains(&e.source.as_str())
                    || justblast_core::domain::Board::from_rows(&e.board).is_err()
            })
        {
            return Err("Invalid cycle history".to_owned());
        }
        Ok(entries)
    }
    pub fn save_history(&self, entries: &[HistoryEntry]) -> Result<(), String> {
        if let Some(root) = &self.root {
            atomic_write(
                &root.join("history.json"),
                &serde_json::to_vec(entries).map_err(|e| e.to_string())?,
            )?;
        }
        Ok(())
    }
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        if let Some(parent) = path.parent() {
            fs::File::open(parent)?.sync_all()?;
        }
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|e| e.to_string())
}

pub fn save_diagnostic(
    store: &Store,
    image: &crate::device::vision::Image,
    reason: &str,
) -> Result<(), String> {
    let Some(root) = store.root() else {
        return Ok(());
    };
    let directory = root.join("diagnostics");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let count = fs::read_dir(&directory)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|e| e == "png"))
        .take(30)
        .count();
    if count >= 30 {
        return Ok(());
    }
    let stem = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, image.width as u32, image.height as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(
                &image
                    .rgb
                    .iter()
                    .flat_map(|p| p.iter().copied())
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| e.to_string())?;
    }
    atomic_write(&directory.join(format!("{stem}.png")), &output)?;
    atomic_write(&directory.join(format!("{stem}.json")),&serde_json::to_vec_pretty(&serde_json::json!({"schemaVersion":1,"width":image.width,"height":image.height,"reason":reason})).map_err(|e|e.to_string())?)
}
