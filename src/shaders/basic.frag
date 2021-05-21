#version 450

layout(location=0) in vec4 i_colour;
layout(location=0) out vec4 o_colour;

void main() {
	o_colour = i_colour;
}