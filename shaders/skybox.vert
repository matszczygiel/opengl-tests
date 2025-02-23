#version 330 core
layout (location = 0) in vec3 position;

out vec3 tex_coord;

uniform mat4 projection;
uniform mat4 view;

void main() {
    tex_coord = position;
    vec4 pos = projection * mat4(mat3(view)) * vec4(position, 1.0);
    gl_Position = pos.xyww;
}  