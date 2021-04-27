#version 450

layout(location=0) in vec3 v_color;
layout(location=1) in vec2 v_tex_coords;
layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;


void main() {
    vec4 texColor = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    if(texColor.a < 0.1)
        discard;
    f_color = texColor;
}