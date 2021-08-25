use winit::window::Window;

/// This helps initialize the renderer, so all these don't have to be passed around seperately
pub struct RendererBase {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
}

impl RendererBase {
    /// Creates the surface, device, queue, and swapchain for a Window.
    pub async fn create(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::TEXTURE_COMPRESSION_BC,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let (swap_chain_descriptor, swap_chain) =
            Self::create_swap_chain(&surface, &device, size.width, size.height);

        RendererBase {
            surface,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
        }
    }

    pub(crate) fn create_swap_chain(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::SwapChainDescriptor, wgpu::SwapChain) {
        // TODO: Query something better from winit or wgpu?
        let format = wgpu::TextureFormat::Bgra8Unorm;

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            format,
            height,
            width,
            present_mode: wgpu::PresentMode::Mailbox,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        };

        let swap_chain = device.create_swap_chain(surface, &swap_chain_descriptor);

        (swap_chain_descriptor, swap_chain)
    }
}