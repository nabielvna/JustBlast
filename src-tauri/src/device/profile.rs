use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub board: [u32; 4],
    pub slots: [[u32; 4]; 3],
    pub preview_pitch: f64,
    pub cell_majority: f64,
}
impl Screen {
    pub fn tray(&self) -> [u32; 4] {
        let x = self.slots.iter().map(|r| r[0]).min().unwrap_or(0);
        let y = self.slots.iter().map(|r| r[1]).min().unwrap_or(0);
        [
            x,
            y,
            self.slots.iter().map(|r| r[0] + r[2]).max().unwrap_or(x) - x,
            self.slots.iter().map(|r| r[1] + r[3]).max().unwrap_or(y) - y,
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0
            || self.height == 0
            || self.width > 8192
            || self.height > 8192
            || u64::from(self.width) * u64::from(self.height) * 4 > 25_000_000
            || !self.preview_pitch.is_finite()
            || self.preview_pitch < 4.0
            || !self.cell_majority.is_finite()
            || self.cell_majority <= 0.0
            || self.cell_majority > 1.0
        {
            return Err("Invalid calibrated screen".to_owned());
        }
        for [x, y, w, h] in [self.board, self.slots[0], self.slots[1], self.slots[2]] {
            if w == 0
                || h == 0
                || x.checked_add(w).is_none_or(|v| v > self.width)
                || y.checked_add(h).is_none_or(|v| v > self.height)
            {
                return Err("Calibration rectangle outside screen".to_owned());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Touch {
    pub gain: f64,
    pub lift: f64,
    pub hold: f64,
    pub drop: f64,
    pub gap: f64,
    pub risk_max: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Timing {
    pub tray_wait: f64,
    pub retry_wait: f64,
    pub tray_attempts: u32,
    pub command_timeout: f64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "export-contracts", derive(ts_rs::TS))]
pub struct Profile {
    pub screen: Screen,
    pub touch: Touch,
    pub runtime: Timing,
}
impl Profile {
    pub fn builtin() -> Self {
        Self::parse(include_str!("../../../resources/default-profile.toml"))
            .expect("Validated built-in calibration")
    }
    pub fn parse(raw: &str) -> Result<Self, String> {
        let mut value: toml::Value =
            toml::from_str(raw).map_err(|e| format!("Invalid calibration: {e}"))?;
        if let Some(table) = value.as_table_mut() {
            for key in ["weights", "rule_context", "combo_seed"] {
                table.remove(key);
            }
        }
        let p: Self = value
            .try_into()
            .map_err(|e| format!("Invalid calibration: {e}"))?;
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.screen.validate()?;
        let t = &self.touch;
        let r = &self.runtime;
        if !t.gain.is_finite()
            || t.gain <= 0.0
            || !t.lift.is_finite()
            || !(0.0..=600.0).contains(&t.lift)
            || [t.hold, t.drop, t.gap, t.risk_max]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || t.risk_max < t.hold.max(t.drop).max(t.gap)
            || [r.tray_wait, r.retry_wait, r.command_timeout]
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0 || *v > 120.0)
            || r.tray_attempts == 0
            || r.tray_attempts > 20
        {
            return Err("Invalid timing or touch calibration".to_owned());
        }
        Ok(())
    }
    pub fn export(&self) -> Result<String, String> {
        self.validate()?;
        toml::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
pub fn list(store: &crate::preferences::Store) -> Result<Vec<String>, String> {
    let mut ids = vec!["default".to_owned()];
    if let Some(root) = store.root() {
        let directory = root.join("profiles");
        if directory.exists() {
            let mut custom: Vec<String> = std::fs::read_dir(directory)
                .map_err(|e| e.to_string())?
                .filter_map(Result::ok)
                .filter_map(|e| {
                    let p = e.path();
                    if p.extension()?.to_str()? != "toml" {
                        return None;
                    }
                    let id = p.file_stem()?.to_str()?.to_owned();
                    (valid_id(&id) && id != "default").then_some(id)
                })
                .take(64)
                .collect();
            custom.sort();
            ids.extend(custom);
        }
    }
    Ok(ids)
}
pub fn load(store: &crate::preferences::Store, id: &str) -> Result<Profile, String> {
    if !valid_id(id) {
        return Err("Invalid calibration profile identity".to_owned());
    }
    if id == "default" {
        return Ok(Profile::builtin());
    }
    let path = store
        .root()
        .ok_or("Calibration storage is unavailable")?
        .join("profiles")
        .join(format!("{id}.toml"));
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("Calibration file is too large".to_owned());
    }
    Profile::parse(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
}
