mod base;
mod pbr;
mod text;

use base::{Base,BaseError};


use raw_window_handle::HasRawWindowHandle;
use glam::UVec2;
use thiserror::Error;

pub struct Renderer {
    base: Base,
    text
}

impl Renderer {
    pub async fn new(
        window: impl HasRawWindowHandle,
        size: UVec2,
    ) -> Result<Self, RendererError> {
        let base = Base::new(window, size).await?;

        let mesh_manager = Megabuffer::new(&base)?;
        let texture_manager = TextureManager::new(&base)?;
        let sprite_pipeline = SpritePipeline::new(&base, &mesh_manager, &texture_manager)?;
        let console_renderer = ConsoleRenderer::new();

        Ok(Self {
            base,
            mesh_manager,
            texture_manager,
            sprite_pipeline,
        })
    }
    pub fn render(&mut self) {
        let base = &self.base;

        let frame = match base.surface.get_current_texture() {
            Ok(x) => x,
            Err(err) => {
                log::error!("Dropped frame: {err}");
                return
            }
        };

        let frame_view = frame.texture.create_view(&Default::default());

        let mut encoder = base.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: None,
            });
        }

        base.queue.submit([ encoder.finish() ]);

        frame.present();
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum RendererError {
    #[error("Failed to init base renderer")]
    Base(#[from] BaseError),

    #[error("Failed to init mesh buffers")]
    MeshManager(#[from] MeshManagerError),

    #[error("Failed to init texture buffers")]
    TextureManager(#[from] TextureManagerError),

    #[error("Failed to init sprite pipeline")]
    SpritePipeline(#[from] SpritePipelineError),
}