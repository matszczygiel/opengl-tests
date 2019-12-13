#version 330 core

out vec4 fragment_color;

in vec3 world_position;

uniform samplerCube environmental_map;

const float PI = 3.14159265359;

void main() {
    vec3 N     = normalize(world_position);
    vec3 up    = vec3(0.0, 1.0, 0.0);
    vec3 right = cross(up, N);
    up         = cross(N, right);
    
    vec3 irradiance = vec3(0.0);   
    float sample_delta = 0.025;
    int n_samples = 0;
    for(float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for(float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            vec3 tangent_sample = vec3(sin(theta) * cos(phi),  sin(theta) * sin(phi), cos(theta));
            vec3 world_sample = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * N; 
            irradiance += texture(environmental_map, world_sample).rgb * sin(2.0 * theta) / 2.0;
            n_samples++;
        }
    }
    irradiance = PI * irradiance / float(n_samples);
    fragment_color = vec4(irradiance, 1.0);
}