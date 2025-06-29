#version 300 es
precision highp float;

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texcoord;

out vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}