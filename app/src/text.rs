use cosmic_text::{
    Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache,
    fontdb::{self, Database},
};

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash},
    sync::Arc,
};

use base::{
    text::{Label, TextFamily, TextProperty},
    util::F32Helper,
    zone,
};
use macroquad::prelude::Texture2D;
use macroquad::prelude::WHITE;
use macroquad::prelude::draw_texture;
use std::hash::Hasher;

/// Handles text rendering and caching.
pub struct Texter {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: HashMap<TextCacheKey, TextObject>,
}

impl Texter {
    pub fn new() -> Self {
        zone!();
        // family name: "Blood Cyrillic"
        let blood_cyr = Arc::new(include_bytes!("../../assets/font/BLOODCYR.ttf"));

        // family name: "Noto Sans"
        let noto = Arc::new(include_bytes!("../../assets/font/NotoSans-Regular.ttf"));

        let mut font_db = Database::new();
        font_db.load_font_source(fontdb::Source::Binary(blood_cyr));
        font_db.load_font_source(fontdb::Source::Binary(noto));
        // let _font_ids = font_db.load_font_source(fontdb::Source::Binary(noto));

        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), font_db);

        // A SwashCache stores rasterized glyphs, create one per application
        let swash_cache = SwashCache::new();

        Texter { font_system, swash_cache, cache: HashMap::new() }
    }

    /// Set text for a key.
    pub fn set_text(&mut self, w: f32, h: f32, text: &[(&str, TextProperty)]) -> Label {
        zone!();
        let cache_key = TextCacheKey::new(text, w, h);
        let to = self.cache.entry(cache_key).or_insert_with(|| {
            let buffer = Buffer::new(&mut self.font_system, Metrics::new(33., 40.));
            let mut to = TextObject::new(buffer);
            to.set_text(&mut self.font_system, w, h, text);
            to
        });

        let rect = base::Rect::new_wh(to.width, to.height);
        Label { handle: cache_key.to_handle(), rect }
    }

    /// Draws a text previously set with set_text
    pub fn draw_text(&mut self, handle: u128, x: f32, y: f32) -> Option<base::Rect> {
        zone!();
        let to = self.cache.get_mut(&TextCacheKey::from_handle(handle))?;
        Some(to.draw(&mut self.font_system, &mut self.swash_cache, x, y))
    }

    pub fn collect_garbage(&mut self) {
        zone!();
        self.cache.retain(|_key, to| {
            to.last_drawn += 1;
            to.last_drawn <= 30
        });
    }
}

/// we use this so that we have to allocate strings all the time
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TextCacheKey {
    hash_code: u64,
    total_length: usize,
}

impl TextCacheKey {
    pub fn new(text: &[(&str, TextProperty)], w: f32, h: f32) -> Self {
        zone!();
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        F32Helper(w).hash(&mut hasher);
        F32Helper(h).hash(&mut hasher);
        let hash_code = hasher.finish();
        let total_length = text.iter().map(|(s, _)| s.len()).sum();

        Self { hash_code, total_length }
    }

    pub fn to_handle(&self) -> u128 {
        let Self { hash_code, total_length } = self;
        let high = (*hash_code as u128) << 64;
        let low = *total_length as u128;
        high | low
    }

    pub fn from_handle(handle: u128) -> Self {
        let (hash_code, total_length) = ((handle >> 64) as u64, handle as u64 as usize);
        Self { hash_code, total_length }
    }
}

/// turns the game TextProperty into the cosmic text equivalent
pub fn to_attr(prop: &TextProperty) -> Attrs<'static> {
    zone!();
    let mut attr = Attrs::new();
    if let Some(c) = prop.color_opt {
        attr = attr.color(cosmic_text::Color::rgba(
            (c.r * 255.).round() as u8,
            (c.g * 255.).round() as u8,
            (c.b * 255.).round() as u8,
            (c.a * 255.).round() as u8,
        ));
        // optional inside, but only non optional setter :/
    }
    attr.family(match prop.family {
        TextFamily::BloodCyrillic => fontdb::Family::Name("Blood Cyrillic"),
        TextFamily::NotoSans => fontdb::Family::Name("Noto Sans"),
    })
    .metrics(Metrics {
        font_size: prop.metrics.font_size,
        line_height: prop.metrics.line_height,
    })
}

/// A helper object for rendering and re-rendering a text on changes.
pub struct TextObject {
    buffer: Buffer,
    texture: Option<Texture2D>,
    width: f32,
    height: f32,
    /// used for garbage collection
    last_drawn: u32,
}

impl TextObject {
    fn new(buffer: Buffer) -> Self {
        Self { buffer, texture: None, width: 0., height: 0., last_drawn: 0 }
    }

    // TODO roll into new()
    fn set_text(
        &mut self,
        font_system: &mut FontSystem,
        w: f32,
        h: f32,
        text: &[(&str, TextProperty)],
    ) {
        zone!();
        self.buffer.set_size(font_system, Some(w), Some(h));
        self.buffer.set_rich_text(
            font_system,
            text.iter().map(|(s, a)| (*s, to_attr(a))),
            &Attrs::new(),
            Shaping::Advanced,
            None,
        );
        self.buffer.set_redraw(true);

        let mut width = 0.0;
        let mut height = 0.;
        for run in self.buffer.layout_runs() {
            width = run.line_w.max(width);
            height += run.line_height;
        }

        self.height = height;
        self.width = width;
    }

    fn draw(
        &mut self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        x: f32,
        y: f32,
    ) -> base::Rect {
        zone!();
        // Create a default text color
        const DEFAULT_COLOR: cosmic_text::Color = cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF);

        if self.buffer.redraw() {
            zone!("redraw");
            self.buffer.set_redraw(false);
            let (Some(w), Some(h)) = self.buffer.size() else { panic!("No size defined") };
            let w = w.ceil() as usize;
            let h = h.ceil() as usize;
            let v = vec![0; 4 * w * h];

            let mut image =
                macroquad::prelude::Image { bytes: v, width: w as _, height: h as _ };

            let mut _height = 0.;
            let mut _total_lines = 0;
            let mut width = 0.0;

            for run in self.buffer.layout_runs() {
                _total_lines += 1;
                width = run.line_w.max(width);
                _height += run.line_height;
                for glyph in run.glyphs.iter() {
                    let physical_glyph = glyph.physical((0., 0.), 1.0);

                    let glyph_color = match glyph.color_opt {
                        Some(some) => some,
                        None => DEFAULT_COLOR,
                    };

                    swash_cache.with_pixels(
                        font_system,
                        physical_glyph.cache_key,
                        glyph_color,
                        |x, y, color| {
                            let (r, g, b, a) = color.as_rgba_tuple();
                            let color = macroquad::prelude::Color::from_rgba(r, g, b, a);
                            let x = (physical_glyph.x + x) as u32;
                            let y = (run.line_y as i32 + physical_glyph.y + y) as u32;
                            if x < image.width as _ && y < image.height as _ {
                                image.set_pixel(x, y, color);
                            }
                        },
                    );
                }
            }
            self.texture = Some(Texture2D::from_image(&image));
        }

        zone!("draw text texture");
        draw_texture(self.texture.as_ref().unwrap(), x, y, WHITE);
        self.last_drawn = 0;

        base::Rect { x, y, w: self.width, h: self.height }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn handle_back_and_forth() {
        let key = TextCacheKey { hash_code: 124414, total_length: u32::MAX as _ };
        let handle = key.to_handle();
        let new_key = TextCacheKey::from_handle(handle);
        assert_eq!(key, new_key);
    }
}
