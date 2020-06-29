#version 450

const vec3 positions[3] = vec3[3](
    vec3(0.0, 0.5, 0.0),
    vec3(-0.5, -0.5, 0.0),
    vec3(0.5, -0.5, 0.0)
);

layout(location=1) in vec3 a_color;

layout(location=0) out vec3 v_color;

void main() {
    v_color = a_color;
    gl_Position = vec4(positions[gl_VertexIndex], 1.0);
}