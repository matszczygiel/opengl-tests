#version 430 core
        
in vec2 uv;
        
out vec3 color;
        
uniform sampler2D texture_samp;

void main(){
  color = texture( texture_samp, uv ).rgb;
}