#version 450

layout(RGBA8) uniform image2D image;

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
void main() {
    ivec2 fc = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y );
    imageStore(image, fc, vec4(vec3(0), 1));
}
