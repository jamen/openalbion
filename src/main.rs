mod renderer;
pub mod pga3d;

use renderer::*;
use pga3d::*;

use std::env;
use std::path::PathBuf;

struct OpenAlbion {}

impl OpenAlbion {
    // fn load_map(&mut self) {

    // }
}

impl Renderer for OpenAlbion {
    fn draw(&mut self, render: &mut Render) {
        let frame = render.swap_chain.get_next_texture().expect("Timeout when acquiring next swap chain texture");

        let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&render.pipeline);
            rpass.draw(0..3, 0..1);
        }

        render.queue.submit(&[ encoder.finish() ]);
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let fable_path = match args.get(0) {
        Some(path) => PathBuf::from(path),
        None => env::current_dir().expect("No Fable directory found.")
    };

    log::debug!("Fable path: {:?}", fable_path);

    let open_albion = OpenAlbion {};

    let orig = e123;
    let px = Mv::point(1.0, 0.0, 0.0);
    let line = orig & px;

    println!("a point       : {:?}", px);
    println!("a line        : {:?}", line);

    open_albion.start()
}