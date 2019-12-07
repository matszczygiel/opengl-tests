#version 330 core
out vec4 fragment_color;

in vec3 tex_coord;

uniform samplerCube skybox;

void main()
{    
    vec3 color = texture(skybox, tex_coord).rgb;
    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2)); 

    fragment_color = vec4(color, 1.0);
}