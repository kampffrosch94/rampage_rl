use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};

const VERTEX: &str = include_str!("shaders/sprite.vert.glsl");

const FRAGMENT: &str = include_str!("shaders/sprite.frag.glsl");

pub struct SpriteShader {
    mat: Material,
}

impl SpriteShader {
    pub fn new() -> Self {
        println!("Loading material");
        let mat = load_material(
            ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("texture_resolution", UniformType::Float2),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::SourceAlpha),
                        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        println!("Material loaded.");
        dbg!(&mat);
        let mat = mat.expect("Shader compilation failed.");

        Self { mat }
    }

    /// reset with gl_use_default_material
    pub fn set(&self, texture: &Texture2D) {
        let Self { mat, .. } = self;
        gl_use_material(mat);
        mat.set_uniform("time", 0. as f32);
        let res: [f32; 2] = [texture.width(), texture.height()];
        mat.set_uniform("texture_resolution", res);
    }
}
