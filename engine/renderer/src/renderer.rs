
use crate::ModelPipelineError;
use crate::base::{Base, RenderPass, BaseError};
use crate::model_pipeline::ModelPipeline;

use raw_window_handle::HasRawWindowHandle;

#[derive(Debug)]
pub enum RendererError {
    Base(BaseError),
    ModelPipeline(ModelPipelineError),
}

pub struct Renderer {
    base: Base,
    model_pipeline: ModelPipeline,
}

struct FrameState {
    frame: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new<W: HasRawWindowHandle>(
        window: &W,
        size: [u32; 2],
    ) -> Result<Self, RendererError> {
        let base = Base::new(window, size).await.map_err(RendererError::Base)?;
        let model_pipeline = ModelPipeline::new(&base).map_err(RendererError::ModelPipeline)?;

        Ok(Self {
            base,
            model_pipeline,
        })
    }

    pub fn resize(&self, size: [u32; 2]) {

    }

    // TODO: Some way to populate the renderer with graphics objects
    // pub fn add_quads()
    // pub fn add_mesh()
    // pub fn submit_job()

    // TODO: Make it so the resource in use are determined each render, and unused resources get garbage collected. Resources that are unused but should be kept warm for a future render should also be passed.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.base.surface.get_current_texture()?;

        let view = frame.texture.create_view(&Default::default());

        let frame_state = FrameState {
            frame,
            view,
        };

        let cmd_bufs = [
            self.main_render_pass(&frame_state)
        ];

        self.base.queue.submit(cmd_bufs);

        frame_state.frame.present();

        Ok(())
    }

    fn main_render_pass(
        &mut self,
        frame_state: &FrameState,
    ) -> wgpu::CommandBuffer {
        let base = &self.base;
        let view = &frame_state.view;

        let mut encoder = base.device.create_command_encoder(&Default::default());

        {
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

            rpass.set_pipeline(&self.model_pipeline.pipeline);
        }

        encoder.finish()
    }
}