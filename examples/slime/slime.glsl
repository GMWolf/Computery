#version 450
#define PI 3.1415926538
struct Particle {
    vec2 pos;
    float dir;
    int type;
};

layout(std430) buffer Particles {
    Particle particles[];
};

layout(r32f) uniform image2D trails;

float random (vec2 st) {
    return fract(sin(dot(st.xy,
    vec2(12.9898,78.233)))*
    43758.5453123);
}

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() {
    uint id = gl_GlobalInvocationID.x;
    Particle particle = particles[id];

    vec2 fieldSize = vec2(imageSize( trails ).xy);

    // Move
    vec2 vd = vec2(cos(particle.dir), sin(particle.dir));
    vec2 newPos = particle.pos + vd;
    if (clamp(newPos, vec2(0), fieldSize - 1.0) == newPos) {
        particle.pos = newPos;
        float t = imageLoad(trails, ivec2(particle.pos + 0.5)).r;


        imageStore(trails, ivec2(particle.pos + 0.5), vec4(t + particle.type));
    } else {
        particle.dir = (2 * PI * random(vec2(id, 2.6)) - PI);
    }

    //Change dir
    float sensorAngle = PI * 0.5;

    float sense_distance = 20;

    vec2 leftSensePos = particle.pos + vec2(cos(particle.dir + sensorAngle), sin(particle.dir + sensorAngle)) * sense_distance;
    float ls = imageLoad(trails, ivec2(leftSensePos + 0.5)).r * particle.type;
    vec2 rightSensePos = particle.pos + vec2(cos(particle.dir - sensorAngle), sin(particle.dir - sensorAngle)) * sense_distance;
    float rs = imageLoad(trails, ivec2(rightSensePos + 0.5)).r * particle.type;
    vec2 forwardSensePos = particle.pos + vd * sense_distance;
    float fs = imageLoad(trails, ivec2(forwardSensePos + 0.5)).r * particle.type;

    float turnRad = 0.1;


    if (fs > ls && fs > ls) {

    } else if (fs < ls && fs < rs) {
       particle.dir += turnRad * (2.0 * random(particle.pos) - 1.0);
    } else if (fs < ls) {
        particle.dir += turnRad;
    } else if (rs < ls) {
        particle.dir -= turnRad;
    }

    // write out new particle
    particles[id] = particle;
}
