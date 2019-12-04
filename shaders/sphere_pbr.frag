#version 330

in vec2 uv;
in vec3 world_position;
in vec3 world_normal;

out vec4 fragment_color;


uniform vec3 albedo;
uniform float metallic;
uniform float roughness;
uniform float ao;

uniform vec3 world_cam_posiiton;

#define LIGHT_COUNT 4

uniform vec3 light_positions[LIGHT_COUNT];
uniform vec3 light_colors[LIGHT_COUNT];

const float PI = 3.14159265359;

float normal_distribution_ggx(float n_dot_h, float roughness) {
    float r_sq = roughness * roughness;
    float bracket = n_dot_h * n_dot_h * (r_sq - 1.0) + 1.0;
    return r_sq / (PI * bracket* bracket);
}

float gemoetry_funciton_schlick_ggx(float dot_prod, float k){
    return dot_prod / (dot_prod * (1.0 - k) + k);
}

float geometry_funciton_smith(float n_dot_v, float n_dot_l, float k) {
    float ggx1 = gemoetry_funciton_schlick_ggx(n_dot_v, k);
    float ggx2 = gemoetry_funciton_schlick_ggx(n_dot_l, k);
    return ggx1 * ggx2;
}

vec3 fresnel_schlick(float h_dot_v, vec3 F0){
    return F0 + (1.0 - F0) * pow(1.0 - h_dot_v, 5.0);
}

void main() {
    vec3 N = normalize(world_normal);
    vec3 V = normalize(world_cam_posiiton - world_position);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    float k = (roughness + 1.0) * (roughness + 1.0) / 8.0;

    vec3 Lo = vec3(0.0);
    for(int i =0; i < LIGHT_COUNT; ++i) {
        vec3 L = normalize(light_positions[i] - world_position);
        vec3 H  = normalize(V + L);

        float light_distance = length(light_positions[i] - world_position);
        float attenuation = 1.0 / (light_distance);
        vec3 radiance = light_colors[i] * attenuation;
    
        vec3 f_lambert = albedo / PI;

        float n_dot_v = max(dot(N, V), 0.0);
        float n_dot_h = max(dot(N, H), 0.0);
        float n_dot_l = max(dot(N, L), 0.0);
        float h_dot_v = max(dot(H, V), 0.0);

        float n_specular = normal_distribution_ggx(n_dot_h, roughness);
        float d_specular = geometry_funciton_smith(n_dot_v, n_dot_l, k);
        vec3 ks = fresnel_schlick(h_dot_v, F0);

        float denom = 4.0 * n_dot_h * n_dot_l;
        float f_cook_torrance = n_specular * d_specular / max(denom, 0.001);

        vec3 kd = vec3(1.0) - ks;
        kd *= (1.0 - metallic);

        Lo += (kd * f_lambert + ks * f_cook_torrance) * radiance * n_dot_l;
    }

    vec3 ambient = vec3(0.03) * albedo * ao;

    vec3 color = ambient + Lo;

    color /= (color + vec3(1.0));

    color = pow(color, vec3(1.0/2.2));
    
    fragment_color = vec4(color, 1.0);
}