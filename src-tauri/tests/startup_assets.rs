// Startup asset regression tests.
//
// Root cause this guards against: the Tauri context macro embeds the bundle
// icons listed in `tauri.conf.json` at compile time; the 32x32 PNG becomes
// the default window icon's raw RGBA buffer. A corrupt or empty icon file
// decodes to zero pixels, and at runtime Tauri validates it during window
// creation inside the macOS `did_finish_launching` callback
// (a non-unwinding FFI boundary), so the panic aborts the process with
// EXC_CRASH/SIGABRT before the first window appears:
//
//   invalid icon: The specified dimensions (32x32) don't match the number
//   of pixels supplied by the `rgba` argument (0). Expected 1024.
//
// These tests decode every referenced icon and assert real pixel data, so
// a bad asset fails `cargo test` instead of aborting startup.

use std::path::{Path, PathBuf};

fn src_tauri_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Icon paths exactly as referenced by `tauri.conf.json` `bundle.icon`.
fn bundle_icons() -> Vec<String> {
    let raw = std::fs::read_to_string(src_tauri_dir().join("tauri.conf.json"))
        .expect("tauri.conf.json must be readable");
    let config: serde_json::Value = serde_json::from_str(&raw).expect("tauri.conf.json must parse");
    config
        .get("bundle")
        .and_then(|b| b.get("icon"))
        .and_then(|i| i.as_array())
        .expect("bundle.icon must be an array")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

/// Decode a PNG and return (width, height, decoded byte count).
fn decode_png(path: &Path) -> (u32, u32, usize) {
    let file = std::fs::File::open(path).expect("icon must open");
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().expect("icon PNG header must parse");
    let (width, height) = {
        let info = reader.info();
        (info.width, info.height)
    };
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf).expect("icon PNG must decode");
    assert_eq!(
        frame.buffer_size(),
        buf.len(),
        "decoded frame must fill the reported buffer"
    );
    (width, height, frame.buffer_size())
}

/// Expected edge length from names like `32x32.png` or `128x128@2x.png`.
fn expected_edge(name: &str) -> Option<u32> {
    let stem = name.trim_end_matches(".png");
    let (base, scale) = match stem.split_once('@') {
        Some((base, suffix)) => (base, suffix.trim_end_matches('x').parse::<u32>().ok()?),
        None => (stem, 1),
    };
    let edge: u32 = base.split_once('x')?.1.parse().ok()?;
    Some(edge * scale)
}

#[test]
fn every_bundle_icon_exists() {
    let icons = bundle_icons();
    assert!(!icons.is_empty(), "tauri.conf.json must list bundle icons");
    for icon in &icons {
        let path = src_tauri_dir().join(icon);
        assert!(
            path.is_file(),
            "bundle icon {icon} referenced by tauri.conf.json must exist on disk"
        );
    }
}

#[test]
fn bundle_png_icons_decode_to_real_pixels() {
    for icon in bundle_icons() {
        if !icon.ends_with(".png") {
            continue;
        }
        let path = src_tauri_dir().join(&icon);
        let (width, height, bytes) = decode_png(&path);

        // The exact validation that aborted startup: decoded RGBA length
        // must equal width * height (RGBA8 => 4 bytes per pixel).
        assert!(
            bytes >= (width as usize) * (height as usize),
            "{icon} decoded to {bytes} bytes for {width}x{height}; \
             a zero-pixel decode panics during window creation"
        );
        assert!(bytes > 0, "{icon} must decode to a non-empty buffer");

        if let Some(expected) = expected_edge(&icon) {
            assert_eq!(
                width, expected,
                "{icon} width must match its declared dimension"
            );
            assert_eq!(
                height, expected,
                "{icon} height must match its declared dimension"
            );
        }
    }
}

#[test]
fn default_window_icon_has_the_expected_pixel_count() {
    // Tauri uses the 32x32 bundle icon as the default window icon. The
    // runtime requires a pixel count of 32 * 32 = 1024, i.e. an RGBA
    // buffer of 1024 * 4 = 4096 bytes; a zero-pixel decode aborts startup.
    let path = src_tauri_dir().join("icons/32x32.png");
    let (width, height, bytes) = decode_png(&path);
    assert_eq!((width, height), (32, 32));
    assert_eq!(
        (width as usize) * (height as usize),
        1024,
        "pixel count must be 1024"
    );
    assert_eq!(
        bytes, 4096,
        "default window icon must supply 4096 RGBA bytes"
    );
}

#[test]
fn png_icons_contain_visible_content() {
    // A truncated or stub PNG can parse yet decode to fully transparent
    // data; require actual non-zero pixel content in the primary icons.
    for icon in ["icons/32x32.png", "icons/128x128.png", "icons/icon.png"] {
        let path = src_tauri_dir().join(icon);
        let file = std::fs::File::open(&path).expect("icon must open");
        let mut reader = png::Decoder::new(file)
            .read_info()
            .expect("icon PNG header must parse");
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).expect("icon PNG must decode");
        assert!(
            buf.iter().any(|&b| b != 0),
            "{icon} must contain non-zero pixel content"
        );
    }
}

#[test]
fn icns_bundle_icon_is_present_and_valid_container() {
    // icon.icns is referenced by tauri.conf.json for the macOS bundle; a
    // missing or stub file breaks packaging even when startup succeeds.
    let path = src_tauri_dir().join("icons/icon.icns");
    assert!(path.is_file(), "icons/icon.icns must exist");
    let bytes = std::fs::read(&path).expect("icon.icns must read");
    assert!(bytes.len() > 16, "icon.icns must not be a stub file");
    assert_eq!(
        &bytes[0..4],
        b"icns",
        "icon.icns must start with the icns magic"
    );
}
