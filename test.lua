
local tex = texture {
    width = display_width;
    height = display_height;
    format = rgba8;
}


pass {
    shader = "shader.glsl";
    entry = "main";
    bindings = {
        image = tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    }
}

pass {
    shader = "shader2.glsl";
    entry = "main";
    bindings = {
        image = tex;
    };
    num_threads = {
        x = display_width;
        y = display_height;
        z = 1;
    }
}