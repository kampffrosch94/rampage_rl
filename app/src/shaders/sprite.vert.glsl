#version 300 es
precision highp float;

in vec3 position;
in vec2 texcoord;

out vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}