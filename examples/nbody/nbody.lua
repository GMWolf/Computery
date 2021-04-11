

local particle_count = 1000;

local tex = texture {
    width = display_width;
    height = display_height;
    format = rgba8;
}


local positions = buffer {
    size = 8 * particle_count;
}

local velocities = buffer {
    size = 8 * particle_count;
}


init_pass {
    shader = "init_particles.glsl";
    entry = "main";
    bindings = {
        ParticlePositions = positions;
        ParticleVelocities = velocities;
    };
    num_threads = {
        x = particle_count;
        y = 1;
        z = 1;
    };
}


pass {
    shader = "clear.glsl";
    entry = "main";
    bindings = {
        image = tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    };
}

pass {
    shader = "nbody.glsl";
    entry = "main";
    bindings = {
        ParticlePositions = positions;
        ParticleVelocities = velocities;
    };
    num_threads = {
        x = particle_count;
        y = 1;
        z = 1;
    }
}

pass {
    shader = "dots.glsl";
    entry = "main";
    bindings = {
        ParticlePositions = positions;
        image = tex;
    };
    num_threads = {
        x = particle_count;
        y = 1;
        z = 1;
    }
}

display {
    image = tex;
}