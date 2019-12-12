#version 330 core

out vec4 fragment_color;

in vec3 world_position;

uniform samplerCube environmental_map;
uniform int env_map_resolution;
uniform float roughness;

const float PI = 3.14159265359;

float normal_distribution_ggx(float n_dot_h, float roughness) {
    float a = roughness * roughness;
    float bracket = n_dot_h * n_dot_h * ( a * a - 1.0) + 1.0;
    return a * a / (PI * bracket* bracket);
}

// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// efficient VanDerCorpus calculation.
float radical_inverse_van_der_corpus(uint bits) {
     bits = (bits << 16u) | (bits >> 16u);
     bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
     bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
     bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
     bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
     return float(bits) * 2.3283064365386963e-10; // / 0x100000000
}

vec2 hammersley_set(uint i, uint N) {
	return vec2(float(i)/float(N), radical_inverse_van_der_corpus(i));
}

vec3 importance_sample_ggx(vec2 x_i, vec3 N, float roughness) {
	float a = roughness * roughness;
	float phi = 2.0 * PI * x_i.x;
	float cos_theta = sqrt((1.0 - x_i.y) / (1.0 + (a*a - 1.0) * x_i.y));
	float sin_theta = sqrt(1.0 - cos_theta * cos_theta);
	
	vec3 H;
	H.x = cos(phi) * sin_theta;
	H.y = sin(phi) * sin_theta;
	H.z = cos_theta;
	
	vec3 up        = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
	vec3 tangent   = normalize(cross(up, N));
	vec3 bitangent = cross(N, tangent);
	
	vec3 sample_vec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sample_vec);
}

void main() {		
    vec3 N = normalize(world_position);
    vec3 R = N;
    vec3 V = R;

    const uint SAMPLE_COUNT = 1024u;
    vec3 prefiltered_color = vec3(0.0);
    float total_weight = 0.0;
    for(uint i = 0u; i < SAMPLE_COUNT; ++i) {
        vec2 x_i = hammersley_set(i, SAMPLE_COUNT);
        vec3 H   = importance_sample_ggx(x_i, N, roughness);
        vec3 L   = normalize(2.0 * dot(V, H) * H - V);

        float n_dot_l = max(dot(N, L), 0.0);
        float n_dot_h = max(dot(N, H), 0.0);
        float h_dot_v = max(dot(H, V), 0.0);
        
        if(n_dot_l > 0.0) {
            float D   = normal_distribution_ggx(n_dot_h, roughness);
            float pdf = D * n_dot_h / (4.0 * h_dot_v) + 0.0001; 

            float sa_texel  = 4.0 * PI / (6.0 * env_map_resolution * env_map_resolution);
            float sa_sample = 1.0 / (float(SAMPLE_COUNT) * pdf + 0.0001);
            float mip_level = roughness == 0.0 ? 0.0 : 0.5 * log2(sa_sample / sa_texel); 

            prefiltered_color += textureLod(environmental_map, L, mip_level).rgb * n_dot_l;
            total_weight      += n_dot_l;
        }
    }

    prefiltered_color /= total_weight;

    fragment_color = vec4(prefiltered_color, 1.0);
}