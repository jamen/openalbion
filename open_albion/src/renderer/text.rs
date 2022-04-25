use std::collections::HashMap;
use std::mem;

use fontdue::layout::{CoordinateSystem, GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings, Metrics};

use etagere::{AllocId, AtlasAllocator, Rectangle};
use glam::Vec2;
use range_alloc::RangeAllocator;

use crate::renderer::Base;

const INCONSOLATA_TTF: &[u8] = include_bytes!("../../assets/font/Inconsolata-Regular.ttf");

const POSITION_SIZE: usize = mem::size_of::<Vec2>();
const UV_SIZE: usize = mem::size_of::<Vec2>();

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    // Atlas
    texture_view: wgpu::TextureView,
    atlas_alloc: AtlasAllocator,
    glyph_cache: HashMap<GlyphRasterConfig, (Metrics, AllocId, Rectangle)>,

    // Mesh
    position_buffer: wgpu::Buffer,
    uv_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_alloc: RangeAllocator<usize>,
    index_alloc: RangeAllocator<usize>,
}

impl TextRenderer {
    pub fn new(base: &Base) -> Self {
        let sprite_vert_module = unsafe {
            const SPRITE_VERT_SPIRV: &[u32] =
                vk_shader_macros::include_glsl!("src/shaders/text.vert");

            base.device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: SPRITE_VERT_SPIRV.into(),
                })
        };

        let sprite_frag_module = unsafe {
            const SPRITE_FRAG_SPIRV: &[u32] =
                vk_shader_macros::include_glsl!("src/shaders/text.frag");

            base.device
                .create_shader_module_spirv(&wgpu::ShaderModuleDescriptorSpirV {
                    label: None,
                    source: SPRITE_FRAG_SPIRV.into(),
                })
        };

        let bind_group_layout =
            base.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Sprite  Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    }],
                });

        let layout = base
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = base
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Sprite Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &sprite_vert_module,
                    entry_point: "main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: POSITION_SIZE as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: UV_SIZE as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &sprite_frag_module,
                    entry_point: "main",
                    targets: &[base.preferred_format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        let max = base.device.limits().max_texture_dimension_2d.min(8192);
        let imax: i32 = max.try_into().unwrap();

        let alloc = AtlasAllocator::new([imax, imax].into());

        let texture = base.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture Atlas"),
            size: wgpu::Extent3d {
                width: max,
                height: max,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // TODO: I dont know if this is right or not
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let texture_view = texture.create_view(&Default::default());

        let font = Font::from_bytes(INCONSOLATA_TTF, FontSettings::default()).unwrap();
        let font_size = 16.0;
        let glyph_cache = HashMap::new();

        Self {
            pipeline,
            bind_group_layout,
            alloc,
            texture_view,
            font_size,
            font,
            glyph_cache,
        }
    }
}
