#version 450

layout(r32f) uniform image2D trails;
layout(r32f) uniform image2D trailsDiffuse;

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
void main() {
    ivec2 coord = ivec2(gl_GlobalInvocationID.xy);
    imageStore(trails, coord, imageLoad(trailsDiffuse, coord));
}
