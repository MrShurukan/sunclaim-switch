use alloc::collections::BTreeMap;
use alloc::string::String;
use core::f32::consts::PI;
use core::str::FromStr;
use libm::sinf;
use crate::raylib::Texture;

/// Helper struct to modulate a value using a sine wave.
/// from and to specify bounds, while period specifies how often it should pulse in seconds
///
/// See [SinePulser::new]
pub struct SinePulser {
    frequency_rad: f32,
    half_diff: f32,
    midpoint: f32,
}

impl SinePulser {
    pub const fn new(from: f32, to: f32, period: f32) -> Self {
        let frequency_rad = if period == 0.0 {
            0.0
        } else {
            (2.0*PI) / period
        };

        let half_diff = (to - from) / 2.0;

        SinePulser {
            frequency_rad,
            half_diff,
            midpoint: from + half_diff,
        }
    }

    pub fn pulse(&self, time: f32) -> f32 {
        self.midpoint + sinf(self.frequency_rad * time) * self.half_diff
    }
}

/// Helper struct to hold a texture atlas (like a sprite sheet) alongside with JSON data about it
/// (only supports Aseprite). Later helps easily draw parts of the image by the key, defined
/// with Shift + C (Slice Tool).
///
/// [K] has to implement [FromStr], as the keys specified by the aseprite will be used to construct
/// your own keys (to avoid string allocations and whatnot)
pub struct AsepriteSheet<K>
where K : FromStr {
    texture: Texture,
    sections: BTreeMap<K, SpriteSheetSection>,
}

impl<K> AsepriteSheet<K> {
    pub const fn new(texture: Texture, json: &str) -> Self {
        
    }
}

#[derive(Debug, Copy, Clone)]
struct SpriteSheetSection {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
