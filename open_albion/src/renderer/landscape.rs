pub struct Landscape<'a> (Pass<'a>);

impl Landscape<'_> {
    pub fn new<'a>(renderer: &'a Renderer) -> Landscape<'a> {
        let pipeline_layout = renderer.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&renderer.bind_groups[0].1],
            push_constant_ranges: &[],
        });

        let pipeline = renderer.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader_module!(renderer.device, "shaders/shader.vert", kind: vert),
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shader_module!(renderer.device, "shaders/shader.frag", kind: frag),
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: renderer.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Subtract,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: mem::size_of::<Vec4>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float4]
                    }
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Landscape (
            Pass {
                bind_groups: vec![&renderer.bind_groups[0].0],
                pipeline,
                draw_data: None,
            }
        )
    }

    pub fn update(&mut self, renderer: &Renderer) {
        let width = lev.header.width as usize;
        let height = lev.header.height as usize;

        let mut vertices = Vec::with_capacity(height * width);
        let mut indices = Vec::with_capacity(height * width * 6);

        for (i, cell) in lev.heightmap_cells.iter().enumerate() {
            let column = i % width;
            let row = i / height;

            vertices.push(Vec4::new(column as f32, cell.height * 2048.0, row as f32, 1.0));

            if column < width - 1 && row < height {
                indices.push(i as u16);
                indices.push((i + 1) as u16);
                indices.push((i + width) as u16);

                // TODO: When textured
                // indices.push((i + width) as u16);
                // indices.push((i + 1) as u16);
                // indices.push((i + width + 1) as u16);
            }
        }

        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("landscape_vertices"),
            usage: wgpu::BufferUsage::VERTEX,
            contents: bytemuck::cast_slice(vertices.as_ref()),
        });

        let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("landscape_indices"),
            usage: wgpu::BufferUsage::INDEX,
            contents: bytemuck::cast_slice(indices.as_ref()),
        });
    }

    pub fn draw(&self, rpass: &mut wgpu::RenderPass) {
        if let Some(draw_data) = self.0.draw_data {
            rpass.set_pipeline(&self.0.pipeline);
            rpass.set_bind_group(0, &self.0.bind_groups[0], &[]);
            rpass.set_index_buffer(draw_data.index_buffer.unwrap().slice(..));
            rpass.set_vertex_buffer(0, draw_data.vertex_buffer.slice(..));
            rpass.draw_indexed(0..draw_data.draw_count as u32, 0, 0..1);
        }
    }
}