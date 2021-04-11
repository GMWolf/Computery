

local particle_count = 14000;

local display_tex = texture {
    width = display_width;
    height = display_height;
    format = rgba8;
}

local trails_tex = texture {
    width = display_width;
    height = display_height;
    format = r32f;
}

local trails_diffuse_tex = texture {
    width = display_width;
    height = display_height;
    format = r32f;
}

local particles = buffer {
    size = 16 * particle_count;
}

init_pass {
    shader = "clear_trails.glsl";
    entry = "main";
    bindings = {
        image = trails_tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    };
}


init_pass {
    shader = "particles_init.glsl";
    entry = "main";
    bindings = {
        Particles = particles;
        trails = trails_tex;
    };
    num_threads = {
        x = particle_count;
        y = 1;
        z = 1;
    };
}

pass {
    shader = "slime.glsl";
    entry = "main";
    bindings = {
        trails = trails_tex;
        Particles = particles;
    };
    num_threads = {
        x = particle_count;
        y = 1;
        z = 1;
    }
}

pass {
    shader = "diffusion.glsl";
    entry = "main";
    bindings = {
        trails = trails_tex;
        trailsDiffuse = trails_diffuse_tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    }
}

pass {
    shader = "copy_trails.glsl";
    entry = "main";
    bindings = {
        trails = trails_tex;
        trailsDiffuse = trails_diffuse_tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    }
}


pass {
    shader = "render_trails.glsl";
    entry = "main";
    bindings = {
        trails = trails_tex;
        image = display_tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    }
}

display {
    image = display_tex;
}