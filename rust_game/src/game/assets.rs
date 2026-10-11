use crate::raylib::TextureError;

pub struct Assets {
    pub 
}

impl Assets {
    pub fn load() -> Result<Self, TextureError> {
        Ok(Self {
            // TODO: Load your assets from romfs
            // Texture::load(c"romfs:/...")?
        })
    }
}
