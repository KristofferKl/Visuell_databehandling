#version 430 core

in  vec4 vert_color;
in  vec3 vert_normal;

out vec4 color;

vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    
    vec4 tempcolor = vert_color*max(0.0, dot(vert_normal, -lightDirection));
    color = vec4(tempcolor.x, tempcolor.y, tempcolor.z, 1.0f);
    
}