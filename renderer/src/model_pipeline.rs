use crate::{base::RenderPass, Base};

#[derive(Debug)]
pub enum ModelPipelineError {}

pub(crate) struct ModelPipeline {
    // pipeline: wgpu::RenderPipeline,
}

impl ModelPipeline {
    pub(crate) fn new(base: &Base) -> Result<ModelPipeline, ModelPipelineError> {
        Ok(Self {})
    }
}

impl RenderPass for ModelPipeline {
    fn render_pass(
        &mut self,
        base: &Base,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _frame: &wgpu::SurfaceTexture,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
    }
}
