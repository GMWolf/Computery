#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location=0) out vec4 f_color;

uniform texture2D tex;
uniform sampler s;
layout(location = 0) in vec2 texCoord;

void main() {
    f_color = vec4(texture( sampler2D(tex, s), texCoord));
}