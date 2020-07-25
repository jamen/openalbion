#version 450

layout(location = 0) in vec4 vert_position;
layout(location = 1) in vec2 vert_color;

layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform Locals {
    mat4 camera_matrix;
};

void main() {
    gl_Position = camera_matrix * vert_position;
    frag_color = vec4(1.0, 0.0, 0.0, 1.0);
}
