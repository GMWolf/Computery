#![feature(control_flow_enum)]

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use std::mem::swap;
use winit::event::VirtualKeyCode::N;
use std::borrow::Cow;
use shaderc::TargetEnv;
use wgpu::{ShaderStage, BindGroupLayoutEntry, StorageTextureAccess, TextureFormat};
use std::num::NonZeroU32;
use spirv_cross::{spirv, glsl};
use std::collections::HashMap;
use rlua::Lua;
use std::fs::read_to_string;
use std::any::Any;
use std::cell::RefCell;
use std::ops::{DerefMut, Deref};
use itertools::Itertools;
use pollster::block_on;

fn compile_shader(source: &str, shader_kind: shaderc::ShaderKind, filename: &str, entry: &str) -> Vec<u32> {
    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_auto_bind_uniforms(true);

    let binary_result = compiler.compile_into_spirv(
        source, shader_kind, filename, entry, Some(&options)
    ).unwrap();

    binary_result.as_binary().to_vec()
}

fn reflect_bind_group( source: &[u32] ) -> (Vec<wgpu::BindGroupLayoutEntry>, HashMap<String, u32>) {
    let ast = spirv_cross::spirv::Ast::<glsl::Target>::parse(&spirv_cross::spirv::Module::from_words(source)).unwrap();
    let resources = ast.get_shader_resources().unwrap();
    
    let mut entries = vec![];

    let mut bindings = HashMap::new();

    for resource in resources.sampled_images {
        let binding = ast.get_decoration(resource.id, spirv::Decoration::Binding).unwrap();
        bindings.insert(resource.name, binding);
        entries.push(BindGroupLayoutEntry {
            binding,
            visibility: ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: Default::default(),
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false
            },
            count: None
        });
    }
    
    for resource in resources.separate_samplers {
        let binding = ast.get_decoration(resource.id, spirv::Decoration::Binding).unwrap();
        bindings.insert(resource.name, binding);
        entries.push(BindGroupLayoutEntry {
            binding,
            visibility: ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Sampler {
                filtering: true,
                comparison: false
            },
            count: None
        })
    }

    for resource in resources.storage_images {

        println!("{:?}",ast.get_decoration(resource.id, spirv::Decoration::Component).unwrap());

        let binding = ast.get_decoration(resource.id, spirv::Decoration::Binding).unwrap();
        bindings.insert(resource.name, binding);
        entries.push(BindGroupLayoutEntry{
            binding,
            visibility: ShaderStage::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: StorageTextureAccess::ReadWrite,
                format: TextureFormat::Rgba8Unorm,
                view_dimension: Default::default()
            },
            count: None
        })
    }


    (entries, bindings)

}


struct ScriptTexture {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

enum ScriptResources {
    Texture(ScriptTexture),
}

struct ScriptPass {
    shader: String,
    entry: String,
    bindings: HashMap<String, usize>,
    x_threads: u32,
    y_threads: u32,
    z_threads: u32,
}

struct ScriptContext {
    resources: Vec<ScriptResources>,
    passes: Vec<ScriptPass>,
}


fn int_to_format(int: u32) -> Option<wgpu::TextureFormat> {
    match int {
        17 => Some(wgpu::TextureFormat::Rgba8Unorm),
        _ => None
    }
}


fn lua_texture(ctx: &mut ScriptContext, def: &rlua::Table) -> usize {

    let width = def.get("width").unwrap();
    let height = def.get("height").unwrap();
    let format = int_to_format(def.get("format").unwrap()).unwrap();

    ctx.resources.push(ScriptResources::Texture(
        ScriptTexture{
            width,
            height,
            format,
        }
    ));

    return ctx.resources.len() - 1;
}


fn lua_pass(ctx: &mut ScriptContext, def: &rlua::Table) {

    let shader = def.get("shader").unwrap();
    let entry = def.get("entry").unwrap();
    let num_threads : rlua::Table = def.get("num_threads").unwrap();
    let x_threads = num_threads.get("x").unwrap();
    let y_threads = num_threads.get("y").unwrap();
    let z_threads = num_threads.get("z").unwrap();

    let mut bindings = HashMap::new();

    let bindingTable : rlua::Table = def.get("bindings").unwrap();
    for a in bindingTable.pairs() {
        let a = a.unwrap();
        bindings.insert(a.0, a.1);
    };

    ctx.passes.push(ScriptPass{shader, entry, bindings, x_threads, y_threads, z_threads});
}

fn parse_lua(source : &str) -> rlua::Result<ScriptContext>{
    let lua = Lua::new();

    let scriptCtx = RefCell::new(ScriptContext {
        resources: vec![],
        passes: vec![],
    });


    lua.context(|lua_ctx| {
       let globals = lua_ctx.globals();

        globals.set("rgba8", wgpu::TextureFormat::Rgba8Unorm as usize);

        lua_ctx.scope(|scope| {
            globals.set(
                "texture",
                scope.create_function_mut(|_, table: rlua::Table| {
                    let mut ctx = scriptCtx.borrow_mut();
                    Ok(lua_texture(ctx.deref_mut(), &table))
                })?,
            )?;

            globals.set(
                "pass",
                scope.create_function_mut(|_, table: rlua::Table|{
                    let mut ctx = scriptCtx.borrow_mut();
                    Ok(lua_pass(ctx.deref_mut(), &table))
                })?,
            )?;

            globals.set("display_width", 1280)?;
            globals.set("display_height", 720)?;

            lua_ctx.load(source).exec()
        })?;

        Ok(())
    })?;

    Ok(scriptCtx.into_inner())
}


struct Shader {
    name: String,
    module: wgpu::ShaderModule,
    compute_pipeline: wgpu::ComputePipeline,
    bindings: HashMap<String, u32>,
}

struct Pass {
    shader_index: usize,
    bind_group: wgpu::BindGroup,
    x_threads: u32,
    y_threads: u32,
    z_threads: u32,
}

fn run(event_loop: EventLoop<()>, window: Window) {

    let lua = read_to_string("test.lua").unwrap();
    let ctx = parse_lua(&lua).unwrap();


    let size = window.inner_size();

    // Pick a GPU
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY | wgpu::BackendBit::BROWSER_WEBGPU);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = block_on(instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
    ).expect("Failed to find adapter");

    let (device, queue) = block_on(adapter
        .request_device(
            &wgpu::DeviceDescriptor{
                label: None,
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                limits: wgpu::Limits::default()
            },
            None,
        )
    ).expect("Failed to create device");


    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_format = adapter.get_swap_chain_preferred_format(&surface);


    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage:: RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };


    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let vs =  compile_shader(include_str!("shader.vert"), shaderc::ShaderKind::Vertex, "shader.vert", "main");
    let fs =  compile_shader(include_str!("shader.frag"), shaderc::ShaderKind::Fragment, "shader.frag", "main");

    let vertex_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(&vs)),
        flags: wgpu::ShaderFlags::VALIDATION,
    });

    let fragment_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(&fs)),
        flags: wgpu::ShaderFlags::VALIDATION,
    });



    let resources : Vec<_> = ctx.resources.iter().map(|res| {
        match res {
            ScriptResources::Texture(tex_def) => {
                let tex = device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: tex_def.width,
                        height: tex_def.height,
                        depth: 1
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: tex_def.format,
                    usage: wgpu::TextureUsage::STORAGE | wgpu::TextureUsage::SAMPLED,
                });
                let view = tex.create_view(&wgpu::TextureViewDescriptor {
                    label: None,
                    format: Some(tex_def.format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    level_count: NonZeroU32::new(1),
                    base_array_layer: 0,
                    array_layer_count: NonZeroU32::new(1),
                });

                (tex, view)
            }
        }
    }).collect();

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
        label: None,
        address_mode_u: Default::default(),
        address_mode_v: Default::default(),
        address_mode_w: Default::default(),
        mag_filter: Default::default(),
        min_filter: Default::default(),
        mipmap_filter: Default::default(),
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        compare: None,
        anisotropy_clamp: None,
        border_color: None
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: Default::default(),
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: true,
                    comparison: false
                },
                count: None
            },
        ]
    });


    let shaders : Vec<_> = ctx.passes.iter().map(|pass_def| (&pass_def.shader, &pass_def.entry)).unique()
        .map(|(file, entry)|{
        let source = read_to_string(file).unwrap();
        let text = compile_shader(&source, shaderc::ShaderKind::Compute, &source, entry);

        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(&text)),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let (bind_group_entries, bindings) = reflect_bind_group(&text);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &bind_group_entries
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: entry,
        });

       Shader { name: file.clone(), module, compute_pipeline, bindings }
    }).collect();


    let passes : Vec<_> = ctx.passes.iter().map(|pass_def|{

        let (shader_index, shader) = shaders.iter().enumerate().find(|(i, s)|s.name == pass_def.shader).unwrap();

        let entries : Vec<_> = pass_def.bindings.iter().map(|(name, &resource_id)|{
            wgpu::BindGroupEntry {
                binding: shader.bindings[name],
                resource: wgpu::BindingResource::TextureView(&resources[resource_id].1),
            }
        }).collect();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &shader.compute_pipeline.get_bind_group_layout(0),
            entries: &entries
        });

        Pass{
            shader_index,
            bind_group,
            x_threads: pass_def.x_threads,
            y_threads: pass_def.y_threads,
            z_threads: pass_def.z_threads,
        }

    }).collect();

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&resources[0].1),
            },
            wgpu::BindGroupEntry{
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            }
        ]
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            &bind_group_layout
        ],
        push_constant_ranges: &[]
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: "main",
            targets: &[swapchain_format.into()],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
    });

    event_loop.run(move |event, _, control_flow| {

        // Take ownership of resources
        let _ = (&instance, &adapter);

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::RedrawRequested(_) => {
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire frame")
                    .output;

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label:None});

                for pass in &passes {
                    let mut pass_encoder = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: None,
                    });

                    pass_encoder.set_pipeline(&shaders[pass.shader_index].compute_pipeline);
                    pass_encoder.set_bind_group(0, &pass.bind_group, &[]);
                    pass_encoder.dispatch(pass.x_threads, pass.y_threads, pass.z_threads);
                }


                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                        label:None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {},
        }

    });

}


fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    run(event_loop, window);
}
