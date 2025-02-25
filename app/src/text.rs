use cosmic_text::{
    fontdb::{self, Database},
    Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache,
};

use std::{collections::HashMap, sync::Arc};

use macroquad::prelude::draw_texture;
use macroquad::prelude::Texture2D;
use macroquad::prelude::WHITE;

pub struct Texter {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: HashMap<u64, TextObject>,
    default_metrics: Metrics,
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

        Texter {
            font_system,
            swash_cache,
            cache: HashMap::new(),
            default_metrics: Metrics::new(33., 40.),
        }
    }

    pub fn set_text(&mut self, key: u64, w: f32, h: f32, text: &[(&str, TextProperty)]) {
        let to = self.cache.entry(key).or_insert_with(|| {
            let buffer = Buffer::new(&mut self.font_system, self.default_metrics);
            TextObject::new(buffer)
        });
        // entry.0.set_metrics(&mut self font_system, metrics);
        to.set_text(&mut self.font_system, w, h, text);
    }

    pub fn draw_text(&mut self, key: u64, x: f32, y: f32) {
        let to = self.cache.get_mut(&key).unwrap();
        to.draw(&mut self.font_system, &mut self.swash_cache, x, y);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextFamily {
    BloodCyrillic,
    NotoSans,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TextMetrics {
    pub font_size: f32,
    pub line_height: f32,
}

impl TextMetrics {
    pub fn new(font_size: f32, line_height: f32) -> Self {
        Self { font_size, line_height }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TextProperty {
    pub color_opt: Option<base::Color>,
    pub family: TextFamily,
    pub metrics: TextMetrics,
}

impl TextProperty {
    pub fn new() -> Self {
        TextProperty {
            color_opt: None,
            family: TextFamily::NotoSans,
            metrics: TextMetrics::new(33., 40.),
        }
    }

    pub fn family(mut self, family: TextFamily) -> Self {
        self.family = family;
        self
    }

    pub fn metrics(mut self, font_size: u32, line_height: u32) -> Self {
        self.metrics.font_size = font_size as _;
        self.metrics.line_height = line_height as _;
        self
    }

    pub fn color(mut self, color: base::Color) -> Self {
        self.color_opt = Some(color);
        self
    }

    pub fn to_attr(self) -> Attrs<'static> {
        let mut attr = Attrs::new();
        if let Some(c) = self.color_opt {
            attr = attr.color(cosmic_text::Color::rgba(
                (c.r * 255.).round() as u8,
                (c.g * 255.).round() as u8,
                (c.b * 255.).round() as u8,
                (c.a * 255.).round() as u8,
            ));
            // optional inside, but only non optional setter :/
        }
        attr.family(match self.family {
            TextFamily::BloodCyrillic => fontdb::Family::Name("Blood Cyrillic"),
            TextFamily::NotoSans => fontdb::Family::Name("Noto Sans"),
        })
        .metrics(Metrics {
            font_size: self.metrics.font_size,
            line_height: self.metrics.line_height,
        })
    }
}

pub struct TextObject {
    buffer: Buffer,
    texture: Option<Texture2D>,
    last_text: Vec<(String, TextProperty)>,
}

impl TextObject {
    pub fn new(buffer: Buffer) -> Self {
        Self { buffer, texture: None, last_text: Vec::new() }
    }

    pub fn set_text(
        &mut self,
        font_system: &mut FontSystem,
        w: f32,
        h: f32,
        text: &[(&str, TextProperty)],
    ) {
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
                text.iter().map(|(s, a)| (*s, a.to_attr())),
                Attrs::new(),
                Shaping::Advanced,
            );
        }
    }

    pub fn draw(
        &mut self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        x: f32,
        y: f32,
    ) {
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
            println!("Redraw Text");
        }

        draw_texture(self.texture.as_ref().unwrap(), x, y, WHITE);
    }
}
