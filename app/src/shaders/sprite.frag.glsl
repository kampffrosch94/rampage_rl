#version 100
#extension GL_OES_standard_derivatives : enable
precision highp float;

varying vec2 uv;

uniform sampler2D Texture;
uniform vec2 texture_resolution;

void main() {
    vec2 pix = uv * texture_resolution;
    vec2 pix_fract = fract(pix);
    pix.x = floor(pix.x) + 0.5;
    pix.y = floor(pix.y) + 0.5;

    gl_FragColor = texture2D(Texture, pix / texture_resolution) ;
}