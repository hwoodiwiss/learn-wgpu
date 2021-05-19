#version 450

layout(location=0) in vec2 f_position;
layout(location=0) out vec4 f_color;

void main() {
	f_color = vec4(f_position, 0.913, 1.0);
}