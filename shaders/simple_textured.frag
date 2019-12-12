#version 330 core
out vec4 fragment_color;
in vec2 texture_uv;

uniform sampler2D texture_map;

void main() {
    vec3 color = texture(texture_map, texture_uv).rgb;
    
    fragment_color = vec4(color, 1.0);
}