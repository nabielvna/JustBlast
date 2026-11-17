use flate2::{Compression, write::GzEncoder};
use justblast_desktop::device::{
    profile::{Profile, Screen},
    transport::{Adb, Transport, decode_frame, rotation, validate_endpoint},
};
use std::{io::Write, sync::atomic::AtomicBool};
fn screen() -> Screen {
    Screen {
        width: 8,
        height: 8,
        board: [0, 0, 8, 8],
        slots: [[0, 4, 2, 4], [2, 4, 2, 4], [4, 4, 2, 4]],
        preview_pitch: 4.0,
        cell_majority: 0.9,
    }
}
fn raw(rect: [u32; 4], format: u32) -> Vec<u8> {
    let mut raw = b"BBV2".to_vec();
    for value in [8, 8, rect[0], rect[1], rect[2], rect[3], format] {
        raw.extend(value.to_le_bytes());
    }
    raw.extend((0..rect[2] * rect[3] * 4).map(|v| v as u8));
    raw
}
fn gzip(raw: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}
#[test]
fn compressed_capture_rejects_wrong_roi_truncation_oversize_trailing_and_corruption() {
    let screen = screen();
    let rect = screen.tray();
    let correct = raw(rect, 1);
    let image = decode_frame(&gzip(&correct), rect, &screen).unwrap();
    assert_eq!((image.width, image.height), (6, 4));
    assert_eq!(image.rgb[1], [4, 5, 6]);
    let mut wrong = correct.clone();
    wrong[12] = 1;
    assert!(decode_frame(&gzip(&wrong), rect, &screen).is_err());
    assert!(decode_frame(&gzip(&correct[..correct.len() - 1]), rect, &screen).is_err());
    let mut large = correct.clone();
    large.extend(vec![0; 10000]);
    assert!(decode_frame(&gzip(&large), rect, &screen).is_err());
    let mut trailing = gzip(&correct);
    trailing.extend(gzip(b"extra"));
    assert!(decode_frame(&trailing, rect, &screen).is_err());
    let mut corrupt = gzip(&correct);
    let last = corrupt.len() - 8;
    corrupt[last] ^= 1;
    assert!(decode_frame(&corrupt, rect, &screen).is_err());
    assert!(decode_frame(&gzip(&raw(rect, 3)), rect, &screen).is_err());
}
#[test]
fn rotation_formats_keep_default_display_identity_and_reject_conflicts() {
    for text in [
        "SurfaceOrientation: 0",
        "Viewport INTERNAL: displayId=0, uniqueId=local:0, port=0, orientation=0, isActive=[1]",
        "DisplayViewport{type=INTERNAL, valid=true, displayId=0, orientation=0}",
        "DisplayInfo{\"Built-in Screen\", displayId 0, rotation 0, type INTERNAL}",
    ] {
        assert_eq!(rotation(text).unwrap(), Some(0));
    }
    assert_eq!(rotation("Viewport EXTERNAL: displayId=2, orientation=0\nViewport INTERNAL: displayId=0, orientation=1").unwrap(),Some(1));
    assert_eq!(
        rotation("Viewport EXTERNAL: displayId=2, orientation=0").unwrap(),
        None
    );
    assert_eq!(rotation("Permission denied").unwrap(), None);
    assert!(rotation("Viewport INTERNAL: displayId=0, orientation=0\nViewport INTERNAL: displayId=0, orientation=2").is_err());
    assert_eq!(
        rotation("DisplayViewport{valid=false,displayId=0,orientation=1}").unwrap(),
        None
    );
}
#[test]
fn endpoint_and_profile_validation_reject_shell_input_and_invalid_calibration() {
    for valid in ["192.168.1.2:37123", "phone.local:1234", "[::1]:5555"] {
        validate_endpoint(valid).unwrap();
    }
    for invalid in [
        "host:0",
        "host:65536",
        "host:5555;touch /tmp/unsafe",
        "-x:5",
        "host",
        "host:1\n",
    ] {
        assert!(validate_endpoint(invalid).is_err());
    }
    let profile = Profile::builtin();
    profile.validate().unwrap();
    let mut bad = profile.clone();
    bad.touch.gain = 0.0;
    assert!(bad.validate().is_err());
    bad = profile.clone();
    bad.screen.board[2] = u32::MAX;
    assert!(bad.validate().is_err());
}
#[test]
fn cancelled_operations_never_launch_adb() {
    let mut adb = Adb::new("a-nonexistent-test-executable".into());
    let cancel = AtomicBool::new(true);
    let error = adb.discover(&cancel).unwrap_err();
    assert!(error.contains("cancelled"));
}
#[cfg(unix)]
#[test]
fn owned_crop_helper_protocol_handles_both_android_headers_and_rejects_invalid_input() {
    use std::{
        path::PathBuf,
        process::{Command, Stdio},
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let executable =
        std::env::temp_dir().join(format!("justblast-crop-test-{}", std::process::id()));
    let output = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("../resources/native/crop.c"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let screen = screen();
    let rect = screen.tray();
    let args = rect
        .into_iter()
        .chain([8, 8])
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    for header_size in [12, 16] {
        for format in [1u32, 2] {
            let mut input = Vec::new();
            for value in [8u32, 8, format] {
                input.extend(value.to_le_bytes());
            }
            input.resize(header_size, 0);
            input.extend((0..256).map(|i| i as u8));
            let mut child = Command::new(&executable)
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(&input).unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            let image = decode_frame(&gzip(&output.stdout), rect, &screen).unwrap();
            assert_eq!(image.rgb[0], [128, 129, 130]);
            assert_eq!(image.rgb.len(), 24);
        }
    }
    let mut child = Command::new(&executable)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&[0; 20]).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let _ = std::fs::remove_file(executable);
}
