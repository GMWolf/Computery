#version 450

#define PI 3.1415926538

struct Particle {
    vec2 pos;
    float dir;
    int type;
};

layout(r32f) uniform image2D trails;

layout(std430) buffer Particles {
    Particle particles[];
};

float random (vec2 st) {
    return fract(sin(dot(st.xy,
    vec2(12.9898,78.233)))*
    43758.5453123);
}

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() {
    uint id = gl_GlobalInvocationID.x;

    vec2 fieldSize = vec2(imageSize( trails ).xy);

    particles[id].pos.x = (0.5 + (random(vec2(id, 0.6)) - 0.5)* 0.2) * fieldSize.x;
    particles[id].pos.y = (0.5 + (random(vec2(id, 1.6)) - 0.5)* 0.2) * fieldSize.y;

    particles[id].dir = (2 * PI * random(vec2(id, 2.6)) - PI);
    if (random(vec2(id, 3.6)) > 0.5) {
        particles[id].type = -1;
    } else {
        particles[id].type = 1;
    }
}