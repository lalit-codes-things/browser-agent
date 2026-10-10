fn main() {
    // The Tauri context macro embeds `tauri.conf.json` and the bundle icons
    // at compile time (the default window icon becomes raw RGBA data inside
    // the binary). If cargo does not track those assets, a regenerated icon
    // is never recompiled in and stale icon data keeps crashing startup at
    // runtime (invalid icon -> panic inside the event-loop callback ->
    // abort). Track them explicitly so asset regeneration always rebuilds.
    println!("cargo:rerun-if-changed=tauri.conf.json");
    for icon in [
        "icons/32x32.png",
        "icons/64x64.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.png",
        "icons/icon.icns",
        "icons/icon.ico",
    ] {
        println!("cargo:rerun-if-changed={icon}");
    }

    tauri_build::build()
}
