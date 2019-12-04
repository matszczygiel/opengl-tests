#version 330

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec2 vertex_uv;
layout (location = 2) in vec3 vertex_normal;

out vec2 uv;
out vec3 world_position;
out vec3 world_normal;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

void main() {
    uv = vertex_uv;
    world_position = vec3(model * vec4(vertex_position, 1.0));
    world_normal = mat3(model) * vertex_normal;

    gl_Position = projection * view * vec4(world_position, 1.0);
}