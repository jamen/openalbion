#version 450

layout(location = 0) in vec4 vertex_position;
layout(location = 0) out vec4 result;

void main() {
    result = vec4(1.0, 0.0, 0.0, 1.0);
}