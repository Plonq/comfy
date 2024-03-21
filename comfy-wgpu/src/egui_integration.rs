use egui::ClippedPrimitive;

pub struct EguiRenderRoutine {
    pub render_pass: egui_wgpu::Renderer,
    pub screen_descriptor: egui_wgpu::ScreenDescriptor,
    #[allow(dead_code)]
    textures_to_free: Vec<egui::TextureId>,
}

impl EguiRenderRoutine {
    /// Creates a new render routine to render a egui UI.
    ///
    /// Egui will always output gamma-encoded color. It will determine if to do
    /// this in the shader manually based on the output format.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        samples: u32,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        let render_pass =
            egui_wgpu::Renderer::new(device, surface_format, None, samples);

        Self {
            render_pass,
            screen_descriptor: egui_wgpu::ScreenDescriptor {
                size_in_pixels: [width, height],
                pixels_per_point: scale_factor,
            },
            textures_to_free: Vec::new(),
        }
    }

    pub fn resize(
        &mut self,
        new_width: u32,
        new_height: u32,
        new_scale_factor: f32,
    ) {
        self.screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [new_width, new_height],
            pixels_per_point: new_scale_factor,
        };
    }

    pub fn end_frame_and_render(
        &mut self,
        ctx: &egui::Context,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        pixels_per_point: f32,
        // view: &wgpu::TextureView,
        // render_pass: &'a mut wgpu::RenderPass<'a>,
    ) -> Vec<ClippedPrimitive> {
        let egui::FullOutput { shapes, textures_delta, .. } = ctx.end_frame();

        let paint_jobs = ctx.tessellate(shapes, pixels_per_point);

        for id in textures_delta.free {
            self.render_pass.free_texture(&id);
        }

        for (id, image_delta) in textures_delta.set {
            self.render_pass.update_texture(device, queue, id, &image_delta);
        }

        self.render_pass.update_buffers(
            device,
            queue,
            encoder,
            &paint_jobs,
            &self.screen_descriptor,
        );

        paint_jobs
        // self.render_pass.execute(
        //     encoder,
        //     view,
        //     &paint_jobs,
        //     &self.screen_descriptor,
        //     None,
        // );
    }

    // pub fn add_to_graph<'node>(
    //     &'node mut self,
    //     graph: &mut RenderGraph<'node>,
    //     mut input: Input<'node>,
    //     output: RenderTargetHandle,
    // ) {
    //     let mut builder = graph.add_node("egui");
    //
    //     let output_handle = builder.add_render_target_output(output);
    //
    //     let rpass_handle = builder.add_renderpass(RenderPassTargets {
    //         targets: vec![RenderPassTarget {
    //             color: output_handle,
    //             clear: Color::BLACK,
    //             resolve: None,
    //         }],
    //         depth_stencil: None,
    //     });
    //
    //     // We can't free textures directly after the call to `execute_with_renderpass` as it freezes
    //     // the lifetime of `self` for the remainder of the closure. so we instead buffer the textures
    //     // to free for a frame so we can clean them up before the next call.
    //     let textures_to_free = mem::replace(&mut self.textures_to_free, mem::take(&mut input.textures_delta.free));
    //     let pt_handle = builder.passthrough_ref_mut(self);
    //
    //     builder.build(move |pt, renderer, encoder_or_pass, _temps, _ready, _graph_data| {
    //         let this = pt.get_mut(pt_handle);
    //         let rpass = encoder_or_pass.get_rpass(rpass_handle);
    //
    //         for tex in textures_to_free {
    //             this.internal.free_texture(&tex);
    //         }
    //         for (id, image_delta) in input.textures_delta.set {
    //             this.internal
    //                 .update_texture(&renderer.device, &renderer.queue, id, &image_delta)
    //         }
    //
    //         this.internal.update_buffers(
    //             &renderer.device,
    //             &renderer.queue,
    //             input.paint_jobs,
    //             &this.screen_descriptor,
    //         );
    //
    //         this.internal
    //             .execute_with_renderpass(rpass, input.paint_jobs, &this.screen_descriptor);
    //     });
    // }
}
