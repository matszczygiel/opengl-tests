#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texture_coord;

out vec2 texture_uv;

void main() {
    texture_uv = texture_coord;
	gl_Position = vec4(position, 1.0);
}