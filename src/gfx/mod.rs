pub mod camera;
pub mod glyph_context;
pub mod glyph_gfx;
pub mod gpu_context;
pub mod world_renderer;

pub struct GfxContext {
    gpu_context: crate::gfx::gpu_context::GpuContext,
    monospace_glyph_context: crate::gfx::glyph_context::MonospaceGlyphContext,
    next_frame_encoder: wgpu::CommandEncoder,
}

impl GfxContext {
    // TODO: `Option` -> `Result`.
    pub async fn create(window: &winit::window::Window) -> Option<GfxContext> {
        let gpu_context = crate::gfx::gpu_context::GpuContext::create(window).await.unwrap();

        let monospace_glyph_context = crate::gfx::glyph_context::MonospaceGlyphContext::new(
            include_bytes!("../../resources/fonts/FiraMono-Regular.ttf").iter().cloned().collect(),
            (20.0, 20.0),
            &gpu_context,
        )?;

        // Create the command encoder used during initialization.
        let init_encoder = gpu_context.create_command_encoder();

        // Flush the initialization commands on the command queue.
        gpu_context.queue().submit(&[init_encoder.finish()]);

        let next_frame_encoder = gpu_context.create_command_encoder();

        Some(Self {
            gpu_context,
            monospace_glyph_context,
            next_frame_encoder,
        })
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Update our GPU context with the new width and height.
        self.gpu_context.resize(size);
    }

    pub fn render(
        &mut self,
        world: &mut crate::state::world::World,
        world_renderer: &mut world_renderer::WorldRenderer,
        camera: &camera::Camera,
    ) {
        let frame = self.gpu_context.get_next_frame().unwrap();

        world_renderer.render(world, camera, &mut self.monospace_glyph_context);

        let (width, height) = self.gpu_context.size();

        self.monospace_glyph_context.glyph_context.glyph_brush.draw_queued(
            self.gpu_context.device(),
            &mut self.next_frame_encoder,
            &frame.view,
            width,
            height,
        ).unwrap();

        // Pull out the command encoder we have been using to build up this frame. We set up the
        // next frame's encoder at the same time.
        let final_encoder = std::mem::replace(
            &mut self.next_frame_encoder,
            self.gpu_context.create_command_encoder(),
        );

        self.gpu_context.submit_command_encoder(final_encoder);
    }

    pub fn size(&self) -> (u32, u32) {
        self.gpu_context.size()
    }
}
