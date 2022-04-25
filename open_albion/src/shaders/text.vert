#version 440

layout(location = 0) in vec3 i_position;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec3 i_tangent;
layout(location = 3) in vec3 i_uv0;

layout(location = 0) out vec3 o_position;

void main() {
    o_position = i_position;

    gl_Position = vec4(i_position, 1.0);
}