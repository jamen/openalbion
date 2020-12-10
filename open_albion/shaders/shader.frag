#version 450

layout(location = 0) in vec4 vertex_position;
layout(location = 0) out vec4 result;

void main() {
    result = vec4(vertex_position.x / 40.0, vertex_position.y / 40.0, vertex_position.z / 40.0, 1.0);
}