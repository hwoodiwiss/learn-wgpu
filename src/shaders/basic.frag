#version 450

layout(location=0) in vec2 tex_coords;

layout(set=0, binding=0) uniform texture2D tex_diffuse;
layout(set=0, binding=1) uniform sampler sam_diffuse;

layout(location=0) out vec4 o_colour;

void main() {
	o_colour = texture(sampler2D(tex_diffuse, sam_diffuse), tex_coords);
}