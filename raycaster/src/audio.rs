use raylib::prelude::*;
use std::sync::OnceLock;

static AUDIO_DEVICE: OnceLock<RaylibAudio> = OnceLock::new();

/// Initialize the global audio device. Safe to call multiple times.
pub fn init_audio() -> &'static RaylibAudio {
    AUDIO_DEVICE.get_or_init(|| RaylibAudio::init_audio_device().expect("Failed to init audio device"))
}

/// Load a music stream borrowing the global audio device. Returns None on error.
pub fn load_music<'a>(path: &str) -> Option<Music<'a>> {
    let device = init_audio();
    // Try the provided path first
    if let Ok(m) = device.new_music(path) { return Some(m); }

    // fallback attempts: src/audios/<basename>, assets/audios/<basename>
    if let Some(name) = std::path::Path::new(path).file_name().and_then(|s| s.to_str()) {
        let alt1 = format!("src/audios/{}", name);
        if let Ok(m) = device.new_music(&alt1) { return Some(m); }
        let alt2 = format!("assets/audios/{}", name);
        if let Ok(m) = device.new_music(&alt2) { return Some(m); }
    }
    eprintln!("Failed to load music {} (tried fallbacks)", path);
    None
}

/// Update a music stream (call every frame)
pub fn update_music(m: &Music) { m.update_stream(); }

/// Play a music stream (expects a Music created from the global device)
pub fn play_music(m: &Music) { m.play_stream(); }

/// Stop/cleanup a music stream
pub fn stop_music(m: &Music) { m.stop_stream(); }

pub fn maybe_step(_rl: &RaylibHandle) {
    // placeholder to be implemented: play step SFX when moving
}
