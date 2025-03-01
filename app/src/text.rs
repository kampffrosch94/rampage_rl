use cosmic_text::{
    fontdb::{self, Database},
    Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache,
};

use std::{collections::HashMap, sync::Arc};

use base::text::{TextFamily, TextProperty};
use macroquad::prelude::draw_texture;
use macroquad::prelude::Texture2D;
use macroquad::prelude::WHITE;

pub struct Texter {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: HashMap<u64, TextObject>,
}

impl Texter {
    pub fn new() -> Self {
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

    pub fn set_text(
        &mut self,
        key: u64,
        w: f32,
        h: f32,
        text: &[(&str, TextProperty)],
    ) -> base::Rect {
        let to = self.cache.entry(key).or_insert_with(|| {
            let buffer = Buffer::new(&mut self.font_system, Metrics::new(33., 40.));
            TextObject::new(buffer)
        });
        // entry.0.set_metrics(&mut self font_system, metrics);
        to.set_text(&mut self.font_system, w, h, text)
    }

    pub fn draw_text(&mut self, key: u64, x: f32, y: f32) -> Option<base::Rect> {
        let to = self.cache.get_mut(&key)?;
        Some(to.draw(&mut self.font_system, &mut self.swash_cache, x, y))
    }
}

pub fn to_attr(prop: &TextProperty) -> Attrs<'static> {
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

pub struct TextObject {
    buffer: Buffer,
    texture: Option<Texture2D>,
    last_text: Vec<(String, TextProperty)>,
    width: f32,
    height: f32,
}

impl TextObject {
    fn new(buffer: Buffer) -> Self {
        Self { buffer, texture: None, last_text: Vec::new(), width: 0., height: 0. }
    }

    fn set_text(
        &mut self,
        font_system: &mut FontSystem,
        w: f32,
        h: f32,
        text: &[(&str, TextProperty)],
    ) -> base::Rect {
        if text.len() != self.last_text.len()
            || text
                .iter()
                .zip(self.last_text.iter())
                // current != last
                .any(|((s, a), (ls, la))| s != ls || a != la)
        {
            self.last_text = text.iter().map(|(s, a)| (s.to_string(), a.clone())).collect();
            self.buffer.set_size(font_system, Some(w), Some(h));
            self.buffer.set_rich_text(
                font_system,
                text.iter().map(|(s, a)| (*s, to_attr(a))),
                Attrs::new(),
                Shaping::Advanced,
            );
            self.buffer.set_redraw(true);

            let mut height = 0.;
            let mut width = 0.0;

            for run in self.buffer.layout_runs() {
                width = run.line_w.max(width);
                height += run.line_height;
            }
            self.height = height;
            self.width = width;
        }
        base::Rect::new_wh(self.width, self.height)
    }

    fn draw(
        &mut self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        x: f32,
        y: f32,
    ) -> base::Rect {
        // Create a default text color
        const DEFAULT_COLOR: cosmic_text::Color = cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF);

        if self.buffer.redraw() {
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

        draw_texture(self.texture.as_ref().unwrap(), x, y, WHITE);

        base::Rect { x, y, w: self.width, h: self.height }
    }
}
