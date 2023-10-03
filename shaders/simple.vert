#version 430 core

in vec3 position;
in vec4 color;
in vec3 normal;

out vec4 vert_color;
out vec3 vert_normal;

layout(location= 10) uniform mat4 comMat;


void main()
{
    gl_Position = comMat*vec4(position, 1.0f);
    vert_color = color;
    vert_normal = normal;
}