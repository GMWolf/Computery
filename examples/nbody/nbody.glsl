#version 450

layout(std430) coherent buffer ParticlePositions {
    vec2 positions[];
};

layout(std430) coherent buffer ParticleVelocities {
    vec2 velocities[];
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() {
    uint id = gl_GlobalInvocationID.x;

    positions[id] += velocities[id];

    for(int i = 0; i < 1000; i++) {

        if (i != id) {

            vec2 d = positions[id] - positions[i];
            float d2 = max(dot(d, d), 0.01);

            vec2 f = normalize(-d) / d2;

            velocities[id] += f * 0.0000001;
        }

    }

}
