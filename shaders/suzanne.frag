#version 330 core
        
in vec2 uv;
in vec3 positon_worldspace;
in vec3 normal_cameraspace;
in vec3 eyedir_cameraspace;
in vec3 lightdir_cameraspace;
        
out vec3 color;
        
uniform sampler2D texture_samp;
uniform vec3 light_pos_worldspace;
uniform vec3 light_color;
uniform float light_power;

void main(){

    vec3 material_diffuse_color = texture(texture_samp, uv).rgb;
    vec3 material_ambient_color = 0.1 * material_diffuse_color;
    vec3 material_specular_color = vec3(0.3, 0.3, 0.3);

    float dist = length(light_pos_worldspace - positon_worldspace);

    vec3 n = normalize(normal_cameraspace);
    vec3 l = normalize(lightdir_cameraspace);
    float cos_theta = clamp(dot(n, l), 0.0, 1.0);

    vec3 eye = normalize(eyedir_cameraspace);
    vec3 r = reflect(-l, n);
    float cos_alpha = clamp(dot(eye, r), 0.0, 1.0);

    color = 
    material_ambient_color + 
    material_diffuse_color * light_color * light_power * cos_theta / (dist * dist) +
    material_specular_color * light_color * light_power * pow(cos_alpha, 5) / (dist * dist);
}