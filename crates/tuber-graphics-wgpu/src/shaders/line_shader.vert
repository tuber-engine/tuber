#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_color;
layout(location=2) in vec2 a_tex_coords;

layout(location=0) out vec3 v_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    v_color = a_color;
    gl_Position = u_view_proj * vec4(a_position.x, a_position.y, 0.0, 1.0);
}