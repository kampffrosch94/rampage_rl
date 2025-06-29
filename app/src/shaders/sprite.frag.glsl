#version 300 es
precision highp float;

uniform sampler2D Texture;
uniform float time;
uniform vec2 texture_resolution;

in vec2 uv;
out vec4 outColor; 

void main() {
    vec2 pix = 0.5 + (uv * texture_resolution);
    // float scale_factor = sin(time * 0.4) * 2.0;
    // pix = pix  / scale_factor;
    pix = floor(pix) + min(fract(pix) / fwidth(pix), 1.0) - 0.5;
    outColor = texture(Texture, pix/ texture_resolution);
}