use justblast_desktop::device::{
    profile::Profile,
    vision::{self, Image, Slot},
};
use std::{fs::File, io::BufReader, path::Path};
fn image(name: &str) -> Image {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .join(name);
    let mut decoder = png::Decoder::new(BufReader::new(File::open(path).unwrap()));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().unwrap();
    let mut output = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut output).unwrap();
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => panic!("Unsupported fixture pixel type"),
    };
    Image::new(
        info.width as usize,
        info.height as usize,
        output[..info.buffer_size()]
            .chunks(channels)
            .map(|p| [p[0], p[1], p[2]])
            .collect(),
    )
    .unwrap()
}
#[test]
fn saved_tray_failures_keep_identical_catalog_ids() {
    let screen = Profile::builtin().screen;
    for (name, expected) in [
        ("tray-side-glow.png", vec![24, 36, 34]),
        ("tray-textured-background.png", vec![34, 30, 20]),
    ] {
        let tray =
            vision::read_tray(&image(name), &screen, (screen.tray()[0], screen.tray()[1])).unwrap();
        assert!(tray.complete(), "{name}: {tray:?}");
        assert_eq!(
            tray.pieces()
                .iter()
                .map(|p| p.catalog_id)
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
    }
}
#[test]
fn full_snapshot_and_crop_preserve_absolute_touch_boxes() {
    let screen = Profile::builtin().screen;
    let image = image("before.png");
    let snapshot = vision::read_snapshot(&image, &screen).unwrap();
    assert_eq!(snapshot.board.rows()[0], "########");
    assert_eq!(
        snapshot
            .tray
            .pieces()
            .iter()
            .map(|p| p.catalog_id)
            .collect::<Vec<_>>(),
        [14, 14, 20]
    );
    let bbox = snapshot.tray.pieces()[0].bbox;
    assert!((bbox[0] - 144).abs() <= 2 && (bbox[1] - 1764).abs() <= 2);
    assert_eq!([bbox[2], bbox[3]], [162, 108]);
    let cropped = image.crop(screen.tray()).unwrap();
    let tray = vision::read_tray(&cropped, &screen, (screen.tray()[0], screen.tray()[1])).unwrap();
    assert_eq!(tray, snapshot.tray);
}
#[test]
fn textured_board_matches_all_64_reference_cells() {
    let screen = Profile::builtin().screen;
    let snapshot = vision::read_snapshot(&image("board-textured-stars.png"), &screen).unwrap();
    assert_eq!(
        snapshot.board.rows(),
        [
            ".##.....", "####....", "####.##.", ".##.####", "..#.####", ".#...##.", ".#....#.",
            "..#...#."
        ]
        .map(str::to_owned)
    );
    assert_eq!(
        snapshot
            .tray
            .pieces()
            .iter()
            .map(|p| p.catalog_id)
            .collect::<Vec<_>>(),
        [20, 35, 32]
    );
}
#[test]
fn partial_tray_only_omits_an_observed_empty_slot() {
    let screen = Profile::builtin().screen;
    let mut image = image("before.png");
    let [x, y, w, h] = screen.slots[2].map(|v| v as usize);
    for r in y..y + h {
        let pixel = image.rgb[r * image.width + x];
        for c in x..x + w {
            image.rgb[r * image.width + c] = pixel;
        }
    }
    let snapshot = vision::read_snapshot(&image, &screen).unwrap();
    assert!(matches!(snapshot.tray.slots[2], Slot::Empty));
    assert_eq!(snapshot.tray.pieces().len(), 2);
}
#[test]
fn an_empty_tray_and_invalid_dimensions_are_rejected() {
    let screen = Profile::builtin().screen;
    let blank = Image::new(1080, 2400, vec![[40, 80, 120]; 1080 * 2400]).unwrap();
    assert!(vision::read_snapshot(&blank, &screen).is_err());
    assert!(Image::new(2, 2, vec![[0; 3]; 3]).is_err());
    assert!(Profile::parse("[screen]\nwidth=0").is_err());
}

#[test]
fn native_touch_geometry_stays_within_capture_quantization_and_preserves_risk_delays() {
    use justblast_core::{
        domain::{catalog, combo::ComboState},
        solver::{Game, Piece, SearchOptions, solve},
    };
    let profile = Profile::builtin();
    let observation = vision::read_snapshot(&image("before.png"), &profile.screen).unwrap();
    let game = Game {
        checkpoint: None,
        board: observation.board,
        pieces: observation
            .tray
            .pieces()
            .into_iter()
            .map(|p| Piece {
                slot: p.slot,
                catalog_id: Some(p.catalog_id),
                name: "Piece".to_owned(),
                shape: p.shape,
            })
            .collect(),
        states: [ComboState::default(); 3],
        context: Default::default(),
        weights: Default::default(),
    };
    let plan = solve(
        &game,
        &catalog::evaluation_shapes(),
        SearchOptions::default(),
        |_| true,
    )
    .unwrap();
    let (actions, delays) =
        justblast_desktop::device::touch::prepare(&game, &plan, &observation.tray, &profile)
            .unwrap();
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/fixtures/touch-reference.json")).unwrap();
    let expected = reference["actions"].as_array().unwrap();
    assert_eq!(actions.len(), expected.len());
    for (action, expected) in actions.iter().zip(expected) {
        assert_eq!(action.kind, expected[0].as_str().unwrap());
        assert!((i64::from(action.x) - expected[1].as_i64().unwrap()).abs() <= 2);
        assert!((i64::from(action.y) - expected[2].as_i64().unwrap()).abs() <= 2);
    }
    for (index, actual) in delays.iter().enumerate() {
        let extra = if index % 3 == 2
            && plan.steps[index / 3].result.lines() > 0
            && plan.steps[index / 3].result.board.bits() == 0
        {
            justblast_desktop::device::touch::CLEAR_SETTLE_SECONDS
        } else {
            0.0
        };
        assert!((actual - 0.08 - extra).abs() < 1e-12);
    }
    for index in 0..plan.steps.len() {
        let (_, single_delays) = justblast_desktop::device::touch::prepare_range(
            &game,
            &plan,
            &observation.tray,
            &profile,
            index..index + 1,
        )
        .unwrap();
        assert_eq!(single_delays, delays[index * 3..index * 3 + 3]);
    }
}

#[test]
fn unclear_board_keeps_read_cells_and_predicts_ambiguous_cells() {
    let mut cells = vec![
        vision::CellEvidence {
            state: '.',
            surface: 0.0,
            centre: 0.0,
            lower: 0.0,
            foreground: 0.0,
            background: 1.0,
        };
        64
    ];
    cells[0].state = '#';
    cells[1].state = '?';
    cells[1].foreground = 0.6;
    cells[1].background = 0.4;
    cells[2].state = '?';
    assert_eq!(vision::draft_board(&cells).rows()[0], "##......");
    assert_eq!(cells[1].state, '?');
}

#[test]
fn automatic_layout_finds_grid_and_tray_without_saved_coordinates() {
    let image = image("before.png");
    let mut screen = Profile::builtin().screen;
    screen.board = [100, 300, 800, 800];
    screen.slots = [
        [0, 1200, 300, 300],
        [350, 1200, 300, 300],
        [700, 1200, 300, 300],
    ];
    screen.preview_pitch = 70.0;
    let (found, confident) = vision::auto_layout(&image, &screen).unwrap();
    assert!(confident, "{found:?}");
    for (actual, expected) in found.board.into_iter().zip([63, 582, 954, 954]) {
        assert!((actual as i32 - expected).abs() <= 8, "{found:?}");
    }
    let tray = vision::read_tray(&image, &found, (0, 0)).unwrap();
    assert!(tray.complete(), "{found:?}: {tray:?}");
    assert_eq!(
        tray.pieces()
            .iter()
            .map(|p| p.catalog_id)
            .collect::<Vec<_>>(),
        [14, 14, 20]
    );
}

#[test]
fn blank_screen_does_not_claim_automatic_grid_detection() {
    let screen = Profile::builtin().screen;
    let image = Image::new(1080, 2400, vec![[40, 70, 100]; 1080 * 2400]).unwrap();
    let (found, confident) = vision::auto_layout(&image, &screen).unwrap();
    assert!(!confident);
    assert_eq!(found.board, screen.board);
}

#[test]
fn tray_calibration_matches_cell_pitch_and_bounds_for_z_and_large_l_shapes() {
    use justblast_core::domain::Shape;
    let source = image("before.png");
    let mut frame = source.clone();
    let tile = source.crop([514, 1766, 54, 54]).unwrap();
    for y in 1650..2020 {
        for x in 0..1080 {
            frame.rgb[y * 1080 + x] = source.rgb[1660 * 1080 + x];
        }
    }
    let shapes = [
        Shape::new(vec![(0, 1), (0, 2), (1, 0), (1, 1)]).unwrap(),
        Shape::new(vec![(0, 2), (1, 2), (2, 0), (2, 1), (2, 2)]).unwrap(),
        Shape::new(vec![(0, 0), (0, 1), (1, 1), (2, 1)]).unwrap(),
    ];
    let mut origins = Vec::new();
    for (i, shape) in shapes.iter().enumerate() {
        let x = [225, 540, 855][i] - usize::from(shape.width()) * 54 / 2;
        let y = 1818 - usize::from(shape.height()) * 54 / 2;
        origins.push((x, y));
        for &(r, c) in shape.cells() {
            for dy in 0..54 {
                for dx in 0..54 {
                    frame.rgb
                        [(y + usize::from(r) * 54 + dy) * 1080 + x + usize::from(c) * 54 + dx] =
                        tile.rgb[dy * 54 + dx];
                }
            }
        }
    }
    let mut initial = Profile::builtin().screen;
    initial.preview_pitch = 47.0;
    let (found, _) = vision::auto_layout(&frame, &initial).unwrap();
    assert!((found.preview_pitch - 54.0).abs() < 1.6, "{found:?}");
    let tray = vision::read_tray(&frame, &found, (0, 0)).unwrap();
    assert!(tray.complete(), "{found:?}: {tray:?}");
    for piece in tray.pieces() {
        let i = usize::from(piece.slot);
        assert_eq!(piece.shape, shapes[i]);
        assert!(
            (piece.bbox[0] - origins[i].0 as i32).abs() <= 3,
            "{piece:?}"
        );
        assert!(
            (piece.bbox[1] - origins[i].1 as i32).abs() <= 3,
            "{piece:?}"
        );
    }
}

#[test]
fn automatic_layout_preserves_a_working_board_alignment_when_detected_grid_agrees() {
    let image = image("before.png");
    let screen = Profile::builtin().screen;
    assert!(vision::read_snapshot(&image, &screen).is_ok());
    let (found, confident) = vision::auto_layout(&image, &screen).unwrap();
    assert!(confident);
    assert_eq!(found.board, screen.board);
}

fn sparse_calibration_frame(filled: &[(usize, usize)], origin: [u32; 4]) -> Image {
    let mut frame = Image::new(1080, 2400, vec![[65, 75, 100]; 1080 * 2400]).unwrap();
    fn fill(image: &mut Image, [x, y, w, h]: [u32; 4], colour: [u8; 3]) {
        for row in y..y + h {
            for col in x..x + w {
                image.rgb[row as usize * image.width + col as usize] = colour;
            }
        }
    }
    let [x, y, w, h] = origin;
    fill(&mut frame, [x - 5, y - 5, w + 10, h + 10], [12, 14, 27]);
    fill(&mut frame, origin, [20, 23, 41]);
    for r in 0..8 {
        for c in 0..8 {
            let left = x + c * w / 8;
            let right = x + (c + 1) * w / 8;
            let top = y + r * h / 8;
            let bottom = y + (r + 1) * h / 8;
            fill(
                &mut frame,
                [left + 3, top + 3, right - left - 6, bottom - top - 6],
                if filled.contains(&(r as usize, c as usize)) {
                    [150, 185, 8]
                } else {
                    [27, 30, 51]
                },
            );
        }
    }
    frame
}

#[test]
fn board_boundaries_include_empty_outer_rows_columns_and_an_entirely_empty_board() {
    let board = [63, 582, 954, 954];
    let reported = vec![
        (1, 5),
        (1, 6),
        (2, 3),
        (2, 5),
        (4, 0),
        (4, 3),
        (4, 5),
        (4, 6),
        (6, 0),
        (6, 1),
        (6, 3),
        (6, 4),
        (6, 5),
        (6, 7),
        (7, 0),
        (7, 1),
        (7, 3),
        (7, 4),
        (7, 5),
    ];
    for filled in [
        reported.clone(),
        reported.iter().map(|&(r, c)| (7 - r, c)).collect(),
        reported.iter().map(|&(r, c)| (c, r)).collect(),
        reported.iter().map(|&(r, c)| (c, 7 - r)).collect(),
        vec![(3, 3)],
        vec![],
    ] {
        let frame = sparse_calibration_frame(&filled, board);
        let located = vision::locate_board(&frame)
            .unwrap()
            .expect("The whole board frame must be located, even when cells are empty");
        for (actual, expected) in located.into_iter().zip(board) {
            assert!(
                (actual as i32 - expected as i32).abs() <= 8,
                "filled={filled:?}; board={located:?}"
            );
        }
    }
}

#[test]
fn board_detection_replaces_a_profile_shifted_over_an_empty_row() {
    let board = [63, 582, 954, 954];
    let frame = sparse_calibration_frame(
        &[
            (1, 5),
            (1, 6),
            (2, 3),
            (2, 5),
            (4, 0),
            (4, 3),
            (4, 5),
            (4, 6),
            (6, 0),
            (6, 1),
            (6, 3),
            (6, 4),
            (6, 5),
            (6, 7),
            (7, 0),
            (7, 1),
            (7, 3),
            (7, 4),
            (7, 5),
        ],
        board,
    );
    let mut initial = Profile::builtin().screen;
    initial.board[1] += 119;
    let (found, confident) = vision::auto_layout(&frame, &initial).unwrap();
    assert!(confident);
    for (actual, expected) in found.board.into_iter().zip(board) {
        assert!(
            (actual as i32 - expected as i32).abs() <= 8,
            "{:?}",
            found.board
        );
    }
}

#[test]
fn sixteen_coloured_squares_without_a_full_board_are_not_a_calibrated_grid() {
    let board = [63, 582, 954, 954];
    let cells: Vec<_> = (4..8).flat_map(|r| (0..4).map(move |c| (r, c))).collect();
    let source = sparse_calibration_frame(&cells, board);
    let mut frame = Image::new(1080, 2400, vec![[65, 75, 100]; 1080 * 2400]).unwrap();
    for (r, c) in cells {
        let left = board[0] as usize + c * board[2] as usize / 8 + 3;
        let top = board[1] as usize + r * board[3] as usize / 8 + 3;
        for y in top..top + 112 {
            for x in left..left + 112 {
                frame.rgb[y * 1080 + x] = source.rgb[y * 1080 + x];
            }
        }
    }
    assert!(vision::locate_board(&frame).unwrap().is_none());
}
