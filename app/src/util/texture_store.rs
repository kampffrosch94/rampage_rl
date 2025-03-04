use std::collections::HashMap;

use macroquad::texture::{FilterMode, Texture2D, load_texture};
use std::result::Result;

#[derive(Default)]
pub struct TextureStore {
    textures: HashMap<String, Texture2D>,
}

impl TextureStore {
    pub async fn load_texture(
        &mut self,
        path: impl AsRef<str>,
        name: impl Into<String>,
        antialias: bool,
    ) -> Result<(), macroquad::Error> {
        let name = name.into();
        println!("Loaded {name}");
        let texture = load_texture(path.as_ref()).await?;
        if antialias {
            texture.set_filter(FilterMode::Linear);
        } else {
            texture.set_filter(FilterMode::Nearest);
        }
        self.textures.insert(name.into(), texture);
        Ok(())
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<Texture2D> {
        self.textures.get(name.as_ref()).cloned()
    }
}
