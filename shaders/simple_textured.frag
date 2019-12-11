#version 330 core
out vec4 fragment_color;
in vec2 texture_uv;

uniform sampler2D texture_map;

void main() {
    fragment_color = vec4(texture(texture_map, texture_uv).rg, 0.0, 1.0);
}