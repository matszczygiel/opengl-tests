#version 330 core
out vec4 fragment_color;
in vec2 texture_uv;

const float PI = 3.14159265359;

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
	
	vec3 H = vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
	
	vec3 up        = abs(N.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
	vec3 tangent   = normalize(cross(up, N));
	vec3 bitangent = cross(N, tangent);
	
	vec3 sample_vec = tangent * H.x + bitangent * H.y + N * H.z;
	return normalize(sample_vec);
}

float gemoetry_funciton_schlick_ggx(float dot_prod, float k) {
    return dot_prod / (dot_prod * (1.0 - k) + k);
}

float geometry_funciton_smith(float n_dot_v, float n_dot_l, float k) {
    float ggx1 = gemoetry_funciton_schlick_ggx(n_dot_v, k);
    float ggx2 = gemoetry_funciton_schlick_ggx(n_dot_l, k);
    return ggx1 * ggx2;
}

vec2 integrate_brdf(float n_dot_v, float roughness) {
    vec3 V;
    V.x = sqrt(1.0 - n_dot_v*n_dot_v);
    V.y = 0.0;
    V.z = n_dot_v;

    float A = 0.0;
    float B = 0.0; 

    vec3 N = vec3(0.0, 0.0, 1.0);

    float k = roughness * roughness / 2.0;
    
    const uint SAMPLE_COUNT = 1024u;
    for(uint i = 0u; i < SAMPLE_COUNT; ++i) {
        vec2 x_i = hammersley_set(i, SAMPLE_COUNT);
        vec3 H   = importance_sample_ggx(x_i, N, roughness);
        vec3 L   = normalize(2.0 * dot(V, H) * H - V);

        float n_dot_l = max(L.z, 0.0);
        float n_dot_h = max(H.z, 0.0);
        float v_dot_h = max(dot(V, H), 0.0);

        if(n_dot_l > 0.0) {
            float G = geometry_funciton_smith(n_dot_v, n_dot_l, k);
            float G_vis = (G * v_dot_h) / (n_dot_h * n_dot_v);
            float Fc = pow(1.0 - v_dot_h, 5.0);

            A += (1.0 - Fc) * G_vis;
            B += Fc * G_vis;
        }
    }
    A /= float(SAMPLE_COUNT);
    B /= float(SAMPLE_COUNT);
    return vec2(A, B);
}

void main() {
    fragment_color = vec4(integrate_brdf(texture_uv.x, texture_uv.y), 0.0, 1.0);
}