#version 330 core

layout (location = 0) in vec3 vertex_position;

out vec3 world_position;

uniform mat4 projection;
uniform mat4 view;

void main() {
    world_position = vertex_position;  
    gl_Position =  projection * view * vec4(vertex_position, 1.0);
}