#version 450

layout(RGBA8) uniform image2D image;

layout(std430) buffer ParticlePositions {
    vec2 positions[];
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() {

    uint id = gl_GlobalInvocationID.x;

    vec2 pos = positions[id] * vec2(imageSize( image ).xy) + vec2(0.5);

    int radius = 1;



    for(int x = -radius; x <= radius; x++) {
        for(int y = -radius; y <= radius; y++) {
            imageStore(image, ivec2(pos) + ivec2(x, y), vec4(1,1,1, 1));
        }
    }

}
