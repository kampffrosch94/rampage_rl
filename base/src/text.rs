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
    const DEFAULT: Self = TextMetrics { font_size: 33., line_height: 40. };
    pub fn new(font_size: f32, line_height: f32) -> Self {
        Self { font_size, line_height }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TextProperty {
    pub color_opt: Option<crate::Color>,
    pub family: TextFamily,
    pub metrics: TextMetrics,
}

impl TextProperty {
    pub fn new() -> Self {
        TextProperty {
            color_opt: None,
            family: TextFamily::NotoSans,
            metrics: TextMetrics::DEFAULT,
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

    pub fn color(mut self, color: crate::Color) -> Self {
        self.color_opt = Some(color);
        self
    }
}
