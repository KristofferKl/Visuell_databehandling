#version 430 core

in vec3 position;
in vec4 color;
in vec3 normal;

out vec4 vert_color;
out vec3 vert_normal;

layout(location= 10) uniform mat4 model_mat;
layout(location= 26) uniform mat4 view_mat;
mat4 MVP = view_mat * model_mat;


mat3 model_mat3 = mat3(model_mat);
vec3 normal_new = model_mat3 * normal;
vec3 normal_new_norm = normalize(normal_new);


void main()
{


    gl_Position = MVP * vec4(position, 1.0f);
    vert_color = color;
    vert_normal = normal_new_norm;
}