#version 450

layout(std430) buffer ParticlePositions {
    vec2 positions[];
};

layout(std430) buffer ParticleVelocities {
    vec2 velocities[];
};

float random (vec2 st) {
    return fract(sin(dot(st.xy,
    vec2(12.9898,78.233)))*
    43758.5453123);
}

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() {
    uint id = gl_GlobalInvocationID.x;

    positions[id].x = random(vec2(id, 0.6));
    positions[id].y = random(vec2(id, 1.6));

    velocities[id].x = ( 2 * random(vec2(id, 2.6)) - 1) * 0.001;
    velocities[id].y = ( 2 * random(vec2(id, 3.6)) - 1) * 0.001;
}
