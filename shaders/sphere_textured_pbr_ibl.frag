#version 330

in vec2 uv;
in vec3 world_position;
in vec3 world_normal;

out vec4 fragment_color;

uniform sampler2D albedo_map;
uniform sampler2D normal_map;
uniform sampler2D metallic_map;
uniform sampler2D roughness_map;
uniform sampler2D ao_map;

uniform samplerCube irradiance_map;
uniform samplerCube prefiltered_map;
uniform sampler2D brdf_lut;

uniform vec3 world_cam_posiiton;

const float PI = 3.14159265359;

float normal_distribution_ggx(float n_dot_h, float roughness) {
    float a = roughness * roughness;
    float bracket = n_dot_h * n_dot_h * ( a * a - 1.0) + 1.0;
    return a * a / (PI * bracket* bracket);
}

float gemoetry_funciton_schlick_ggx(float dot_prod, float k){
    return dot_prod / (dot_prod * (1.0 - k) + k);
}

float geometry_funciton_smith(float n_dot_v, float n_dot_l, float k) {
    float ggx1 = gemoetry_funciton_schlick_ggx(n_dot_v, k);
    float ggx2 = gemoetry_funciton_schlick_ggx(n_dot_l, k);
    return ggx1 * ggx2;
}

vec3 fresnel_schlick(float cos_theta, vec3 F0){
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}

vec3 get_normal_worldspace() {
    vec3 normal_tangentspace = texture(normal_map, uv).xyz * 2.0 - 1.0;

    vec3 q1  = dFdx(world_position);
    vec3 q2  = dFdy(world_position);
    vec2 st1 = dFdx(uv);
    vec2 st2 = dFdy(uv);

    vec3 N   = normalize(world_normal);
    vec3 T  = normalize(q1 * st2.t - q2 * st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * normal_tangentspace);
}

void main() {
    vec3 N = get_normal_worldspace();
    vec3 V = normalize(world_cam_posiiton - world_position);
    vec3 R = reflect(-V, N);

    vec3 albedo = pow(texture(albedo_map, uv).rgb, vec3(2.2));
    float metallic = texture(metallic_map, uv).r;
    float roughness = texture(roughness_map, uv).r;
    float ao = texture(ao_map, uv).r;

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    float n_dot_v = max(dot(N, V), 0.0);

    vec3 ks = fresnel_schlick(n_dot_v, F0);
    vec3 kd = 1.0 - ks;
    kd *= (1.0 - metallic);

    vec3 irradiance = texture(irradiance_map, N).rgb;
    vec3 diffuse = kd * irradiance * albedo;

    const float MAX_REFLECTION_LOD = 4;
    float lod_level = roughness * MAX_REFLECTION_LOD;
    vec3 prefiltered_color = textureLod(prefiltered_map, R, lod_level).rgb;
    vec2 brdf = texture(brdf_lut, vec2(n_dot_v, roughness)).rg;
    vec3 specular = prefiltered_color * (ks * brdf.x + brdf.y);
    
    vec3 ambient = (diffuse  + specular) * ao;

    vec3 color = ambient;

    color /= (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));
    
    fragment_color = vec4(color, 1.0);
}