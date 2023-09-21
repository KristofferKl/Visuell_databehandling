#version 430 core

in vec3 position;
in vec4 color;

out vec4 vert_color;

layout(location= 3) uniform mat4 comMat;

// //now it takes bot sin (s) and cosine (c)
// layout(location = 3) uniform vec2 sc;
// float s=sc[0];
// float c= sc[1];
// mat4 ATM = mat4(
// vec4(1.0, 0.0, 0.0, 0.0),
// vec4(0.0, c, -s, 0.0),
// vec4(0.0, s, c, 0.0),
// vec4(0.0, 0.0, 0.0, 1.0));





void main()
{
    gl_Position = comMat*vec4(position, 1.0f);
    vert_color = color;
}