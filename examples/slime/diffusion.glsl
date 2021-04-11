#version 450

layout(r32f) uniform image2D trails;
layout(r32f) uniform image2D trailsDiffuse;

float kernel[3][3] = {
{1,2,1},
{2,4,2},
{1,2,1},
};


layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
void main() {

    ivec2 coord = ivec2(gl_GlobalInvocationID.xy);

    vec4 sum = vec4(0);

    vec4 prev = imageLoad(trails, coord);

    for(int x = -1; x <= 1; x++) {
        for(int y = -1; y <= 1; y++) {
            ivec2 s = ivec2(x,y) + coord;
            sum += imageLoad(trails, s) * kernel[x+1][y+1];
        }
    }

    sum /= 16;


    sum = mix(prev, sum, 0.2);

    //sum = max(vec4(0), sum - 0.0001);

    imageStore(trailsDiffuse, coord, sum);


}
