
#version 330 core

layout(location = 0) in vec3 vertex_pos_modelspace;
layout(location = 1) in vec2 vertex_uv;
layout(location = 2) in vec3 vertex_normal_modelspace;

out vec2 uv;
out vec3 positon_worldspace;
out vec3 normal_cameraspace;
out vec3 eyedir_cameraspace;
out vec3 lightdir_cameraspace;

uniform mat4 MVP;
uniform mat4 V;
uniform mat4 M;
uniform vec3 light_pos_worldspace;


void main()
{  
    gl_Position = MVP * vec4(vertex_pos_modelspace, 1.0);
    uv = vertex_uv;

    positon_worldspace = (M * vec4(vertex_pos_modelspace, 1.0)).xyz;
    
    vec3 vertex_pos_cameraspace = (V * M * vec4(vertex_pos_modelspace, 1.0)).xyz;
    eyedir_cameraspace = - vertex_pos_cameraspace;

    vec3 lightpos_cameraspace = (V * vec4(light_pos_worldspace, 1.0)).xyz;
    lightdir_cameraspace = lightpos_cameraspace + eyedir_cameraspace;

    normal_cameraspace = (V * M * vec4(vertex_normal_modelspace, 0.0)).xyz;
}