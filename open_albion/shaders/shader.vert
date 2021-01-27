#version 450

layout(location = 0) in vec4 vert_position;
layout(location = 0) out vec4 vert_position_out;

layout(set = 0, binding = 0) uniform Locals {
    mat4 camera_matrix;
};

void main() {
    vert_position_out = vert_position;
    gl_Position = camera_matrix * vert_position;
}