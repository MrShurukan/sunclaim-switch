use crate::game::aseprite::{AsepriteSheet, AsepriteSheetError, AsepriteSheetKey};
use crate::raylib::TextureError;
use alloc::format;
use alloc::string::String;
use core::ffi::CStr;
use core::fmt::Debug;
use core::str::FromStr;

impl AsepriteSheetKey for Icons {}
#[derive(Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd, strum::EnumString, strum::EnumIter)]
pub enum Icons {
    Energy,
    Metal,
    Action,
    DPad,
    LStick,
    RStick,
    LButton,
    RButton,
    A,
    B,
    X,
    Y,
    ZL,
    ZR,
    MinusButton,
    PlusButton,
}

pub struct Assets {
    pub icons: AsepriteSheet<Icons>,
}

#[derive(Debug, derive_more::From, derive_more::Display)]
pub enum AssetError {
    Texture(TextureError),
    /// Will hold a debug string about the underlying error.
    /// Kind of a hack, cause [AsepriteSheetError] has a generic type
    AsepriteSheet(String),
}

impl Assets {
    pub fn load() -> Result<Self, AssetError> {
        Ok(Self {
            icons: load_aseprite_sheet(c"romfs:/icons/icons.png", c"romfs:/icons/icons.json")?
        })
    }
}

// TODO: Better readable errors for game startup
fn load_aseprite_sheet<K>(texture_path: &CStr, json_path: &CStr) -> Result<AsepriteSheet<K>, String>
where K: Debug + AsepriteSheetKey {
    AsepriteSheet::load(c"romfs:/icons/icons.png", c"romfs:/icons/icons.json")
        .map_err(|e| {
            format!("texture_path: {}; json_path: {};\nerr: {:?}",
                    texture_path.to_str().unwrap(),
                    json_path.to_str().unwrap(),
                    e)
        })
}