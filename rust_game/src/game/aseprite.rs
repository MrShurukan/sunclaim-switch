use crate::raylib::{Color, Frame, Rect, TextFile, TextFileError, Texture, TextureError, Vec2};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::str::FromStr;
use serde::Deserialize;
use AsepriteSheetError::*;

pub trait AsepriteSheetKey: Copy + FromStr + Ord + strum::IntoEnumIterator {}

/// Helper struct to hold a texture atlas (like a sprite sheet) alongside with JSON data about it
/// (only supports Aseprite). Struct helps easily draw parts of the image by the key, defined
/// with Shift + C (Slice Tool).
///
/// [K] has to be an enum. It has to implement [FromStr], because your Aseprite slice names will be used
/// to construct it. It is recommended to use [strum::EnumString] to implement [FromStr] for you.
///
/// [K] also has to implement [Ord] for internal storage and [strum::IntoEnumIterator] for verifying
/// that all keys are covered inside json
pub struct AsepriteSheet<K: AsepriteSheetKey> {
    texture: Texture,
    sections: BTreeMap<K, SpriteSheetSection>,
}

#[derive(Debug, derive_more::From)]
pub enum AsepriteSheetError<K> {
    Texture(TextureError),
    JsonFile(TextFileError),

    JsonStructure(serde_json::error::Error),
    #[from(skip)]
    UnknownSliceNameInJson(String),
    #[from(skip)]
    MultipleFramesAreUnsupported { slice_name: String },
    #[from(skip)]
    DuplicateSliceName(String),
    #[from(skip)]
    MissingEnumVariant(K),
}

impl<K: AsepriteSheetKey> AsepriteSheet<K> {
    pub fn load(texture_path: &CStr, json_path: &CStr) -> Result<Self, AsepriteSheetError<K>> {
        let texture = Texture::load(texture_path)?;
        let json_file = TextFile::load(json_path)?;

        let json = serde_json::from_str::<AsepriteJson>(json_file.text_data())?;
        let mut map = BTreeMap::<K, SpriteSheetSection>::new();

        // Compress the data and extract just the slice name and bounds pairs
        let pairs = json.meta.slices.into_iter()
            .map(|AsepriteSlice { name, keys }| {
                if keys.len() != 1 {
                    return Err(MultipleFramesAreUnsupported { slice_name: name });
                }

                let key = keys.into_iter().next().expect("validated key count");
                Ok((name, key.bounds))
            });

        // Construct the map
        for pair in pairs {
            let (name, bounds) = pair?;

            let Ok(key) = K::from_str(&name) else {
                return Err(UnknownSliceNameInJson(name));
            };

            if map.insert(key, bounds).is_some() {
                return Err(DuplicateSliceName(name));
            }
        }

        // Check that we didn't miss anything (important since we don't return Options later!)
        for key in K::iter() {
            if !map.contains_key(&key) {
                return Err(MissingEnumVariant(key));
            }
        }

        Ok(Self { texture, sections: map })
    }

    /// Draw a sprite from the sheet with a native size (i.e. the same as in the sprite sheet) on
    /// the specified location
    ///
    /// Returns the size of the drawn image
    pub fn draw(&self, frame: &mut Frame, key: K, location: Vec2) -> Vec2 {
        let source = self.sections.get(&key).expect("validated in constructor");

        let dest = (location.x, location.y, source.width as f32, source.height as f32).into();

        frame.texture_pro(&self.texture, (*source).into(), dest, (0, 0).into(), 0f32, Color::WHITE);

        (source.width, source.height).into()
    }

    pub fn draw_resize(&self, frame: &mut Frame, key: K, location: Rect) {
        let source = self.sections.get(&key).expect("validated in constructor");

        frame.texture_pro(&self.texture, (*source).into(), location, (0, 0).into(), 0f32, Color::WHITE);
    }
}

// Modeling this:
// {
//     "meta": {
//         "slices": [{
//             "name": "slice_name",
//             "keys": [{
//                 "bounds": { "x": 100, "y": 200, "w": 32, "h": 32 }
//             }]
//         ]}
//      }
// }

#[derive(Debug, Deserialize)]
struct AsepriteJson {
    meta: AsepriteJsonMeta,
}

#[derive(Debug, Deserialize)]
struct AsepriteJsonMeta {
    slices: Vec<AsepriteSlice>,
}

#[derive(Debug, Deserialize)]
struct AsepriteSlice {
    name: String,
    keys: Vec<AsepriteSliceKey>,
}

#[derive(Debug, Deserialize)]
struct AsepriteSliceKey {
    bounds: SpriteSheetSection,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct SpriteSheetSection {
    x: usize,
    y: usize,
    #[serde(rename="w")]
    width: usize,
    #[serde(rename="h")]
    height: usize,
}

impl From<SpriteSheetSection> for Rect {
    fn from(value: SpriteSheetSection) -> Self {
        (value.x, value.y, value.width, value.height).into()
    }
}