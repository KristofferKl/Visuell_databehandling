#version 430 core

in  vec4 vert_color;
in  vec3 vert_normal;

out vec4 color;

vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    
    color = vert_color*vec4(max(vec3(0.0, 0.0, 0.0), vert_normal*(-lightDirection)), 1.0f);
    
}