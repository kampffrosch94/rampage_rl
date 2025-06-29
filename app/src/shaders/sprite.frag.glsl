#version 100
#extension GL_OES_standard_derivatives : enable
precision highp float;

varying vec2 uv;

uniform sampler2D Texture;
uniform vec2 texture_resolution;

void main() {
    vec2 pix = uv.xy * texture_resolution;
    pix = floor(pix) + min(fract(pix) / fwidth(pix), 1.0) - 0.50;

    gl_FragColor = texture2D(Texture, pix / texture_resolution) ;
}