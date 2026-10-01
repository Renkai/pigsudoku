//! Sound effects shared by the practice games.
//!
//! The WAV files are embedded rather than bundled as assets so that playback
//! works on both the web build and the desktop webview without depending on the
//! asset bundler.

use dioxus::prelude::*;

const CORRECT_WAV: &[u8] = include_bytes!("../assets/sounds/correct.wav");
const WRONG_WAV: &[u8] = include_bytes!("../assets/sounds/wrong.wav");
const COMPLETE_WAV: &[u8] = include_bytes!("../assets/sounds/complete.wav");

pub fn play_correct() {
    play_wav(CORRECT_WAV);
}

pub fn play_wrong() {
    play_wav(WRONG_WAV);
}

pub fn play_complete() {
    play_wav(COMPLETE_WAV);
}

/// Play a WAV through the webview's audio element (works on web and desktop).
fn play_wav(wav: &[u8]) {
    let js = format!(
        "Object.assign(new Audio('data:audio/wav;base64,{}'), {{ volume: 0.8 }}).play().catch(() => {{}});",
        base64_encode(wav)
    );
    spawn(async move {
        let _ = dioxus::document::eval(&js).await;
    });
}

/// Minimal base64 encoder so we don't need an extra dependency.
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encoding_matches_known_values() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn embedded_sound_effects_are_valid_wav_files() {
        for wav in [CORRECT_WAV, WRONG_WAV, COMPLETE_WAV] {
            assert!(wav.len() > 44, "wav too short");
            assert_eq!(&wav[0..4], b"RIFF", "missing RIFF header");
            assert_eq!(&wav[8..12], b"WAVE", "missing WAVE header");
        }
    }
}
