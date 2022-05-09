
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

    // TODO: Some way to populate the renderer with graphics objects
    // pub fn add_quads()
    // pub fn add_mesh()
    // pub fn submit_job()

    // TODO: Make it so the resource in use are determined each render, and unused resources get garbage collected. Resources that are unused but should be kept warm for a future render should also be passed.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let base = &self.base;

        let frame = base.surface.get_current_texture()?;

        let view = frame.texture.create_view(&Default::default());

        let render_passes = [
            &mut self.model_pipeline
        ];

        let cmd_bufs = render_passes.map(|r| {
            let mut encoder = base.device.create_command_encoder(&Default::default());
            r.render_pass(base, &mut encoder, &view, &frame);
            encoder.finish()
        });

        base.queue.submit(cmd_bufs);

        frame.present();

        Ok(())
    }

    pub fn resize(&self, size: [u32; 2]) {

    }
}