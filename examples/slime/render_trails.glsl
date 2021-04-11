#version 450

layout(r32f) uniform image2D trails;
layout(RGBA8) uniform image2D image;
float saturate(float v) { return clamp(v, 0.0,       1.0);       }
vec2  saturate(vec2  v) { return clamp(v, vec2(0.0), vec2(1.0)); }
vec3  saturate(vec3  v) { return clamp(v, vec3(0.0), vec3(1.0)); }
vec4  saturate(vec4  v) { return clamp(v, vec4(0.0), vec4(1.0)); }

vec3 ColorTemperatureToRGB(float temperatureInKelvins)
{
    vec3 retColor;

    temperatureInKelvins = clamp(temperatureInKelvins, 1000.0, 40000.0) / 100.0;

    if (temperatureInKelvins <= 66.0)
    {
        retColor.r = 1.0;
        retColor.g = saturate(0.39008157876901960784 * log(temperatureInKelvins) - 0.63184144378862745098);
    }
    else
    {
        float t = temperatureInKelvins - 60.0;
        retColor.r = saturate(1.29293618606274509804 * pow(t, -0.1332047592));
        retColor.g = saturate(1.12989086089529411765 * pow(t, -0.0755148492));
    }

    if (temperatureInKelvins >= 66.0)
    retColor.b = 1.0;
    else if(temperatureInKelvins <= 19.0)
    retColor.b = 0.0;
    else
    retColor.b = saturate(0.54320678911019607843 * log(temperatureInKelvins - 10.0) - 1.19625408914);

    return retColor;
}


layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
void main() {
    ivec2 fc = ivec2( gl_GlobalInvocationID.x, gl_GlobalInvocationID.y );
    float t = imageLoad(trails, fc).r * 300;

    vec3 c = ColorTemperatureToRGB(t);
    c *= saturate(t/1000);

    imageStore(image, fc, vec4(c, 1));
}
