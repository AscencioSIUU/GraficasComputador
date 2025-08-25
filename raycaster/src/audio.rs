use raylib::prelude::*;

pub fn init_audio() -> RaylibAudio {
    RaylibAudio::init_audio_device().expect("Failed to init audio device")
}

pub fn maybe_step(_rl: &RaylibHandle) {
    // TODO: play step sfx when moving
}
