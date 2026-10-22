use crate::game::aseprite::{AsepriteSheet, AsepriteSheetKey};
use crate::raylib::TextureError;
use alloc::format;
use alloc::string::String;
use core::ffi::CStr;
use core::fmt::Debug;
use game_core::building::BuildingKind;
use game_core::player::PlayerSide;

impl AsepriteSheetKey for Icon {}
#[derive(Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd, strum::EnumString, strum::EnumIter)]
pub enum Icon {
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

impl AsepriteSheetKey for TileSprite {}
#[derive(Debug, Copy, Clone, Ord, Eq, PartialEq, PartialOrd, strum::EnumString, strum::EnumIter)]
pub enum TileSprite {
    ReactorRed,
    ReactorBlue,
    WallLeft,
    WallRight,
    WallDown,
    WallUp,
    MinerRed,
    MinerBlue,
    CursorRed,
    CursorBlue,
}

pub trait TileDrawable {
    fn tile_sprite(&self, player_side: PlayerSide) -> TileSprite;
}

macro_rules! building_tiles {
    ($kind:expr, $side:expr; $(
        $building:ident => ($red:ident, $blue:ident)
    ),* $(,)?) => {
        match ($kind, $side) {
            $(
                (BuildingKind::$building, PlayerSide::Red) =>
                    TileSprite::$red,
                (BuildingKind::$building, PlayerSide::Blue) =>
                    TileSprite::$blue,
            )*
        }
    };
}

impl TileDrawable for BuildingKind {
    fn tile_sprite(&self, player_side: PlayerSide) -> TileSprite {
        building_tiles!(self, player_side;
            Reactor => (ReactorRed, ReactorBlue),
            Miner   => (MinerRed, MinerBlue),
        )
    }
}

pub struct Assets {
    pub icons: AsepriteSheet<Icon>,
    pub tiles: AsepriteSheet<TileSprite>,
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
            icons: load_aseprite_sheet(c"romfs:/icons/icons.png", c"romfs:/icons/icons.json")?,
            tiles: load_aseprite_sheet(c"romfs:/tiles/tiles.png", c"romfs:/tiles/tiles.json")?,
        })
    }
}

// TODO: Better readable errors for game startup
fn load_aseprite_sheet<K>(texture_path: &CStr, json_path: &CStr) -> Result<AsepriteSheet<K>, String>
where K: Debug + AsepriteSheetKey {
    AsepriteSheet::load(texture_path, json_path)
        .map_err(|e| {
            format!("texture_path: {}; json_path: {};\nerr: {:?}",
                    texture_path.to_str().unwrap(),
                    json_path.to_str().unwrap(),
                    e)
        })
}