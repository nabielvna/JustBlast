use super::{profile::Screen, touch::TouchAction, vision::Image};
use flate2::bufread::GzDecoder;
use std::{
    ffi::OsString,
    io::{Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

pub const REMOTE_HELPER: &str = "/data/local/tmp/bb-v2-crop";
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub serial: String,
    pub state: String,
    pub model: String,
}
pub trait Transport: Send {
    fn configure_timeout(&mut self, _timeout: Duration) {}
    fn discover(&mut self, cancel: &AtomicBool) -> Result<Vec<DeviceInfo>, String>;
    fn wireless(
        &mut self,
        address: &str,
        code: Option<&str>,
        cancel: &AtomicBool,
    ) -> Result<(), String>;
    fn connect(
        &mut self,
        serial: &str,
        screen: &Screen,
        helper: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<(), String>;
    fn capture(
        &mut self,
        screen: &Screen,
        tray_only: bool,
        cancel: &AtomicBool,
    ) -> Result<Image, String>;
    fn calibration_capture(
        &mut self,
        _serial: &str,
        _cancel: &AtomicBool,
    ) -> Result<Image, String> {
        Err("Calibration capture is unavailable for this transport".to_owned())
    }
    fn execute(
        &mut self,
        actions: &[TouchAction],
        delays: &[f64],
        force: &AtomicBool,
        on_move: &mut dyn FnMut(usize),
    ) -> Result<(), String>;
    fn close(&mut self);
}
fn spawn_reader(mut reader: impl Read + Send + 'static) -> Receiver<Result<Vec<u8>, String>> {
    let (tx, rx) = mpsc::sync_channel(4);
    std::thread::spawn(move || {
        let mut buffer = [0; 65536];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = tx.send(Ok(Vec::new()));
                    break;
                }
                Ok(n) => {
                    if tx.send(Ok(buffer[..n].to_vec())).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    let _ = tx.send(Err("ADB output channel failed".to_owned()));
                    break;
                }
            }
        }
    });
    rx
}
fn adb_command(executable: &OsString) -> Command {
    let command = Command::new(executable);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut command = command;
        command.creation_flags(0x08000000);
        command
    }
    #[cfg(not(windows))]
    {
        command
    }
}
fn run_process(
    executable: &OsString,
    args: &[OsString],
    input: Option<&[u8]>,
    timeout: Duration,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Vec<u8>, String> {
    if cancel.load(Ordering::Acquire) {
        return Err("Device operation cancelled".to_owned());
    }
    let mut child = adb_command(executable)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "ADB is unavailable. Install Android platform-tools and check PATH.")?;
    let result = (|| {
        let mut stdin = child.stdin.take().ok_or("Missing ADB input channel")?;
        if let Some(input) = input {
            stdin
                .write_all(input)
                .map_err(|_| "ADB input channel failed")?;
        }
        drop(stdin);
        let rx = spawn_reader(child.stdout.take().ok_or("Missing ADB output channel")?);
        let started = Instant::now();
        let mut output = Vec::new();
        let mut ended = false;
        loop {
            if cancel.load(Ordering::Acquire) {
                return Err("Device operation cancelled".to_owned());
            }
            if started.elapsed() > timeout {
                return Err("ADB operation timed out".to_owned());
            }
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(bytes)) => {
                    if bytes.is_empty() {
                        ended = true;
                    } else {
                        if output.len() + bytes.len() > limit {
                            return Err("ADB output exceeded its size limit".to_owned());
                        }
                        output.extend(bytes);
                    }
                }
                Ok(Err(error)) => return Err(error),
                Err(mpsc::RecvTimeoutError::Disconnected) => ended = true,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if ended && let Some(status) = child.try_wait().map_err(|_| "ADB process failed")? {
                if !status.success() {
                    return Err("ADB command failed. Check authorization, connection and helper installation.".to_owned());
                }
                return Ok(output);
            }
        }
    })();
    let _ = child.kill();
    let _ = child.wait();
    result
}
struct Channel {
    child: Child,
    input: ChildStdin,
    output: Receiver<Result<Vec<u8>, String>>,
}
pub struct Adb {
    executable: OsString,
    serial: Option<String>,
    timeout: Duration,
    shell: Option<Channel>,
    release: Option<(i32, i32)>,
    sequence: u64,
}
impl Default for Adb {
    fn default() -> Self {
        Self::new(OsString::from("adb"))
    }
}
impl Adb {
    pub fn new(executable: OsString) -> Self {
        Self {
            executable,
            serial: None,
            timeout: Duration::from_secs(10),
            shell: None,
            release: None,
            sequence: 0,
        }
    }
    fn run(
        &self,
        args: &[&str],
        input: Option<&[u8]>,
        limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, String> {
        run_process(
            &self.executable,
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            input,
            self.timeout,
            limit,
            cancel,
        )
    }
    fn selected(&self) -> Result<&str, String> {
        self.serial
            .as_deref()
            .ok_or_else(|| "Connect a phone first".to_owned())
    }
    fn verify_portrait(&self, cancel: &AtomicBool) -> Result<(), String> {
        for service in ["input", "display"] {
            let text = self.run(
                &["-s", self.selected()?, "shell", "dumpsys", service],
                None,
                262144,
                cancel,
            )?;
            if let Some(rotation) = rotation(&String::from_utf8_lossy(&text))? {
                return if rotation == 0 {
                    Ok(())
                } else {
                    Err("Portrait required. Rotate the phone upright, then Scan again.".to_owned())
                };
            }
        }
        Err("Android screen orientation could not be read".to_owned())
    }
    fn batch(
        &mut self,
        command: &str,
        prefix: &str,
        force: &AtomicBool,
        on_move: &mut dyn FnMut(usize),
    ) -> Result<(), String> {
        if force.load(Ordering::Acquire) {
            return Err("Device operation cancelled".to_owned());
        }
        if self.shell.is_none() {
            let mut child = adb_command(&self.executable)
                .args(["-s", self.selected()?, "shell", "-T"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|_| "Could not open ADB input channel")?;
            let input = child.stdin.take().ok_or("Missing ADB input channel")?;
            let output = spawn_reader(child.stdout.take().ok_or("Missing ADB output channel")?);
            self.shell = Some(Channel {
                child,
                input,
                output,
            });
        }
        self.sequence = self.sequence.wrapping_add(1);
        let marker = format!("BBV2_{}_{}:", std::process::id(), self.sequence);
        let shell = self.shell.as_mut().ok_or("Missing ADB shell")?;
        shell
            .input
            .write_all(format!("{command}; echo {marker}$?\n").as_bytes())
            .and_then(|_| shell.input.flush())
            .map_err(|_| "Input result uncertain: acknowledgement channel disconnected")?;
        let started = Instant::now();
        let mut pending = Vec::new();
        let mut reported = 0;
        loop {
            if force.load(Ordering::Acquire) {
                let _ = shell.child.kill();
                return Err(
                    "Input result uncertain: force stopped. Scan before continuing.".to_owned(),
                );
            }
            if started.elapsed() > self.timeout {
                return Err(
                    "Input result uncertain: acknowledgement timed out. Scan before continuing."
                        .to_owned(),
                );
            }
            match shell.output.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(bytes)) => {
                    if bytes.is_empty() {
                        return Err("Input result uncertain: input shell disconnected".to_owned());
                    }
                    pending.extend(bytes);
                    if pending.len() > 65536 {
                        return Err("Unexpected input acknowledgement output".to_owned());
                    }
                    while let Some(end) = pending.iter().position(|&b| b == b'\n') {
                        let line = String::from_utf8_lossy(&pending[..end])
                            .trim_end_matches('\r')
                            .to_owned();
                        pending.drain(..=end);
                        if let Some(count) = line
                            .strip_prefix(prefix)
                            .and_then(|s| s.parse::<usize>().ok())
                            && count == reported + 1
                        {
                            reported = count;
                            on_move(count);
                        }
                        if let Some(code) = line.strip_prefix(&marker) {
                            return if code == "0" {
                                Ok(())
                            } else {
                                Err("Input result uncertain: remote input command failed. Scan before continuing.".to_owned())
                            };
                        }
                    }
                }
                Ok(Err(error)) => return Err(error),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err("Input result uncertain: input shell disconnected".to_owned());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    }
}
impl Transport for Adb {
    fn configure_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    fn discover(&mut self, cancel: &AtomicBool) -> Result<Vec<DeviceInfo>, String> {
        let output = self.run(&["devices", "-l"], None, 262144, cancel)?;
        let text = String::from_utf8_lossy(&output);
        let mut lines = text.lines();
        if !lines
            .by_ref()
            .any(|s| s.starts_with("List of devices attached"))
        {
            return Err("ADB discovery returned an unexpected response".to_owned());
        }
        Ok(lines
            .filter_map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                if fields.len() < 2 || fields[0].starts_with('*') {
                    return None;
                }
                Some(DeviceInfo {
                    serial: fields[0].to_owned(),
                    state: fields[1].to_owned(),
                    model: fields[2..]
                        .iter()
                        .find_map(|s| s.strip_prefix("model:"))
                        .unwrap_or("")
                        .replace('_', " "),
                })
            })
            .collect())
    }
    fn wireless(
        &mut self,
        address: &str,
        code: Option<&str>,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        validate_endpoint(address)?;
        let output = if let Some(code) = code {
            if code.len() != 6 || !code.bytes().all(|c| c.is_ascii_digit()) {
                return Err("Pairing code must contain six digits".to_owned());
            }
            self.run(
                &["pair", address],
                Some(format!("{code}\n").as_bytes()),
                262144,
                cancel,
            )?
        } else {
            self.run(&["connect", address], None, 262144, cancel)?
        };
        let text = String::from_utf8_lossy(&output);
        if (code.is_some() && !text.contains("Successfully paired"))
            || (code.is_none() && !text.contains("connected to"))
        {
            return Err("Wireless connection failed. Check the port and pairing code.".to_owned());
        }
        Ok(())
    }
    fn connect(
        &mut self,
        serial: &str,
        screen: &Screen,
        helper: Option<&Path>,
        cancel: &AtomicBool,
    ) -> Result<(), String> {
        if serial.is_empty()
            || serial.len() > 200
            || !serial
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.:[]-".contains(&c))
        {
            return Err("Invalid device serial".to_owned());
        }
        if !self
            .discover(cancel)?
            .iter()
            .any(|d| d.serial == serial && d.state == "device")
        {
            return Err("Selected phone is not connected and authorized".to_owned());
        }
        self.serial = Some(serial.to_owned());
        if let Some(helper) = helper {
            let abi = self.run(
                &["-s", serial, "shell", "getprop", "ro.product.cpu.abi"],
                None,
                262144,
                cancel,
            )?;
            if String::from_utf8_lossy(&abi).trim() != "arm64-v8a" {
                return Err("The bundled capture helper requires arm64-v8a".to_owned());
            }
            let path = helper.to_str().ok_or("Invalid helper path")?;
            self.run(
                &["-s", serial, "push", path, REMOTE_HELPER],
                None,
                262144,
                cancel,
            )?;
            self.run(
                &["-s", serial, "shell", "chmod", "700", REMOTE_HELPER],
                None,
                262144,
                cancel,
            )?;
        }
        let version = self.run(
            &["-s", serial, "shell", REMOTE_HELPER, "--version"],
            None,
            262144,
            cancel,
        )?;
        if String::from_utf8_lossy(&version).trim() != "bb-crop-v2-1" {
            return Err("Enable Install capture helper, then Connect again".to_owned());
        }
        let output = self.run(&["-s", serial, "shell", "wm", "size"], None, 262144, cancel)?;
        let text = String::from_utf8_lossy(&output);
        let expression = regex::Regex::new(r"(?:Physical|Override) size: (\d+)x(\d+)")
            .map_err(|e| e.to_string())?;
        let cap = expression
            .captures_iter(&text)
            .last()
            .ok_or("Could not read screen resolution")?;
        if cap[1].parse::<u32>().ok() != Some(screen.width)
            || cap[2].parse::<u32>().ok() != Some(screen.height)
        {
            return Err("Screen resolution does not match the selected calibration".to_owned());
        }
        self.verify_portrait(cancel)
    }
    fn capture(
        &mut self,
        screen: &Screen,
        tray_only: bool,
        cancel: &AtomicBool,
    ) -> Result<Image, String> {
        self.verify_portrait(cancel)?;
        let rect = if tray_only {
            screen.tray()
        } else {
            [0, 0, screen.width, screen.height]
        };
        let [x, y, w, h] = rect;
        let command = format!(
            "screencap | {REMOTE_HELPER} {x} {y} {w} {h} {} {} | gzip -1",
            screen.width, screen.height
        );
        let output = self.run(
            &["-s", self.selected()?, "exec-out", "sh", "-c", &command],
            None,
            26_000_000,
            cancel,
        )?;
        decode_frame(&output, rect, screen)
    }
    fn calibration_capture(&mut self, serial: &str, cancel: &AtomicBool) -> Result<Image, String> {
        if serial.is_empty()
            || serial.len() > 200
            || !serial
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.:[]-".contains(&c))
        {
            return Err("Invalid device serial".to_owned());
        }
        let output = self.run(
            &["-s", serial, "exec-out", "screencap", "-p"],
            None,
            26_000_000,
            cancel,
        )?;
        decode_screenshot(&output)
    }
    fn execute(
        &mut self,
        actions: &[TouchAction],
        delays: &[f64],
        force: &AtomicBool,
        on_move: &mut dyn FnMut(usize),
    ) -> Result<(), String> {
        if actions.is_empty()
            || !actions.len().is_multiple_of(3)
            || delays.len() != actions.len()
            || delays.iter().enumerate().any(|(index, v)| {
                let max = if index % 3 == 2 {
                    1.0 + super::touch::CLEAR_SETTLE_SECONDS
                } else {
                    1.0
                };
                !v.is_finite() || !(0.0..=max).contains(v)
            })
        {
            return Err("Expected complete touch sequences and bounded delays".to_owned());
        }
        self.sequence = self.sequence.wrapping_add(1);
        let prefix = format!("BBSTEP_{}_{}:", std::process::id(), self.sequence);
        let mut commands = Vec::new();
        for (index, action) in actions.iter().enumerate() {
            if action.kind != ["DOWN", "MOVE", "UP"][index % 3]
                || action.x < 0
                || action.y < 0
                || action.x > 10000
                || action.y > 10000
            {
                return Err("Invalid touch sequence".to_owned());
            }
            commands.push(format!(
                "input touchscreen motionevent {} {} {}",
                action.kind, action.x, action.y
            ));
            if action.kind == "UP" {
                commands.push(format!("echo {prefix}{}", (index + 1) / 3));
            }
            if delays[index] > 0.0 && index + 1 < actions.len() {
                commands.push(format!("sleep {:.6}", delays[index]));
            }
        }
        let last = actions.last().ok_or("Missing touch sequence")?;
        self.release = Some((last.x, last.y));
        self.batch(&commands.join(" && "), &prefix, force, on_move)?;
        self.release = None;
        Ok(())
    }
    fn close(&mut self) {
        if let Some(mut shell) = self.shell.take() {
            let _ = shell.child.kill();
            let _ = shell.child.wait();
        }
        if let Some((x, y)) = self.release.take() {
            let cancel = AtomicBool::new(false);
            if let Some(serial) = &self.serial {
                let args: Vec<OsString> = [
                    "-s",
                    serial,
                    "shell",
                    "input",
                    "touchscreen",
                    "motionevent",
                    "UP",
                    &x.to_string(),
                    &y.to_string(),
                ]
                .iter()
                .map(OsString::from)
                .collect();
                let _ = run_process(
                    &self.executable,
                    &args,
                    None,
                    Duration::from_secs(1),
                    262144,
                    &cancel,
                );
            }
        }
    }
}
impl Drop for Adb {
    fn drop(&mut self) {
        self.close();
    }
}
pub fn validate_endpoint(value: &str) -> Result<(), String> {
    let expression =
        regex::Regex::new(r"^(?:[A-Za-z0-9][A-Za-z0-9.\-]*|\[[0-9a-fA-F:]+\]):[0-9]{1,5}$")
            .map_err(|e| e.to_string())?;
    if !expression.is_match(value)
        || value
            .rsplit(':')
            .next()
            .and_then(|s| s.parse::<u16>().ok())
            .is_none_or(|p| p == 0)
    {
        return Err("Use a hostname or IP address with a port in 1..65535".to_owned());
    }
    Ok(())
}
pub fn rotation(text: &str) -> Result<Option<u8>, String> {
    let compile = |pattern: &str| regex::Regex::new(pattern).map_err(|e| e.to_string());
    let mut values = Vec::new();
    let display_zero = compile(r"\bdisplayId\s*=\s*0\b")?;
    let inactive = compile(r"\b(?:valid|isActive)\s*=\s*(?:false|\[0\])")?;
    let orientation = compile(r"\borientation\s*=\s*([0-3])\b")?;
    for block in compile(r"DisplayViewport\{[^}]*\}|Viewport[^\n]*")?.find_iter(text) {
        let block = block.as_str();
        if display_zero.is_match(block)
            && !inactive.is_match(block)
            && let Some(cap) = orientation.captures(block)
        {
            values.push(cap[1].parse::<u8>().map_err(|e| e.to_string())?);
        }
    }
    if values.is_empty() {
        for cap in compile(r"\bSurfaceOrientation\s*:\s*([0-3])\b")?.captures_iter(text) {
            values.push(cap[1].parse::<u8>().map_err(|e| e.to_string())?);
        }
    }
    if values.is_empty() {
        let zero = compile(r"\bdisplayId\s*(?:=\s*)?0\b")?;
        let rotation = compile(r"\brotation\s*(?:=\s*)?([0-3])\b")?;
        for block in compile(r"DisplayInfo\{[^}]*\}")?.find_iter(text) {
            if zero.is_match(block.as_str())
                && let Some(cap) = rotation.captures(block.as_str())
            {
                values.push(cap[1].parse::<u8>().map_err(|e| e.to_string())?);
            }
        }
    }
    if values.iter().any(|v| Some(v) != values.first()) {
        return Err("Screen rotation is changing or ambiguous. Wait, then Scan again.".to_owned());
    }
    Ok(values.first().copied())
}
pub fn decode_frame(packed: &[u8], rect: [u32; 4], screen: &Screen) -> Result<Image, String> {
    screen.validate()?;
    let [x, y, w, h] = rect;
    if w == 0
        || h == 0
        || x.checked_add(w).is_none_or(|v| v > screen.width)
        || y.checked_add(h).is_none_or(|v| v > screen.height)
    {
        return Err("Invalid capture rectangle".to_owned());
    }
    let expected = 32 + (w as usize) * (h as usize) * 4;
    let mut decoder = GzDecoder::new(packed);
    let mut raw = Vec::new();
    (&mut decoder)
        .take(expected as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| "Invalid compressed capture")?;
    if raw.len() != expected || !decoder.get_ref().is_empty() || raw.get(..4) != Some(b"BBV2") {
        return Err("Invalid capture length, header or trailing data".to_owned());
    }
    let mut header = [0; 7];
    for (i, value) in header.iter_mut().enumerate() {
        *value = u32::from_le_bytes(
            raw[4 + i * 4..8 + i * 4]
                .try_into()
                .map_err(|_| "Invalid header")?,
        );
    }
    if header[..6] != [screen.width, screen.height, x, y, w, h] || ![1, 2].contains(&header[6]) {
        return Err("Capture does not match calibrated screen/ROI".to_owned());
    }
    Image::new(
        w as usize,
        h as usize,
        raw[32..].chunks(4).map(|p| [p[0], p[1], p[2]]).collect(),
    )
}

pub fn decode_screenshot(bytes: &[u8]) -> Result<Image, String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let info = reader.info();
    if info.width == 0
        || info.height == 0
        || info.width > 8192
        || info.height > 8192
        || u64::from(info.width) * u64::from(info.height) * 4 > 25_000_000
    {
        return Err("Screenshot dimensions exceed calibration limits".to_owned());
    }
    let mut output = vec![0; reader.output_buffer_size().ok_or("Invalid PNG buffer")?];
    let info = reader.next_frame(&mut output).map_err(|e| e.to_string())?;
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => return Err("Expected a colour screenshot".to_owned()),
    };
    Image::new(
        info.width as usize,
        info.height as usize,
        output[..info.buffer_size()]
            .chunks(channels)
            .map(|p| [p[0], p[1], p[2]])
            .collect(),
    )
}
