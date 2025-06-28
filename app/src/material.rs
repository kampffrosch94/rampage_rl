use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};

const VERTEX: &str = r#"
#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texcoord;

out vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}

"#;

const FRAGMENT: &str = r#"
#version 330 core
precision highp float;

uniform sampler2D Texture;
uniform float time;
uniform vec2 texture_resolution;

varying vec2 uv;

void main() {
    vec2 pix = 0.5 + (uv * texture_resolution);
    // float scale_factor = sin(time * 0.4) * 2.0;
    // pix = pix  / scale_factor;
    pix = floor(pix) + min(fract(pix) / fwidth(pix), 1.0) - 0.5;
    gl_FragColor = texture2D(Texture, pix/ texture_resolution);
}

"#;

pub struct SpriterShader {
    mat: Material,
}

impl SpriterShader {
    pub fn new() -> Self {
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
        )
        .unwrap();

        Self { mat }
    }

    /// reset with gl_use_default_material
    pub fn set(&self, texture: &Texture2D) {
        let Self { mat, .. } = self;
        mat.set_uniform("time", 0. as f32);
        let res: [f32; 2] = [texture.width(), texture.height()];
        mat.set_uniform("texture_resolution", res);
    }
}
