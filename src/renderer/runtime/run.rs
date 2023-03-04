use ash::vk;
use winit::{event::Event, event_loop::EventLoop, platform::run_return::EventLoopExtRunReturn};

use crate::renderer::utilities::MAX_FRAME_DRAWS;

impl<'a> super::Renderer<'a> {
    fn record_command_buffers(&self, img_index: usize) {
        let begin_info = vk::CommandBufferBeginInfo::builder();
        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[img_index])
            .render_area(self.base.surface_extent.into())
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0f32, 0.06, 0.08, 1f32],
                },
            }]);

        unsafe {
            self.base
                .device
                .begin_command_buffer(
                    self.base.command_buffers[self.base.current_frame],
                    &begin_info,
                )
                .expect("Failed to begin recording command buffer");

            self.base.device.cmd_begin_render_pass(
                self.base.command_buffers[self.base.current_frame],
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            self.base.device.cmd_set_scissor(
                self.base.command_buffers[self.base.current_frame],
                0,
                std::slice::from_ref(&self.scissors),
            );

            self.base.device.cmd_set_viewport(
                self.base.command_buffers[self.base.current_frame],
                0,
                std::slice::from_ref(&self.viewport),
            );

            self.base.device.cmd_bind_pipeline(
                self.base.command_buffers[self.base.current_frame],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // self.meshes.iter().enumerate().for_each(|(i, mesh)| {
            //     let vertex_buffers = [mesh.vertex_buffer.buffer];
            //     let offsets = [0];

            //     self.base.device.cmd_bind_vertex_buffers(
            //         self.base.command_buffers[self.base.current_frame],
            //         0,
            //         &vertex_buffers,
            //         &offsets,
            //     );

            //     self.base.device.cmd_bind_descriptor_sets(
            //         self.base.command_buffers[self.base.current_frame],
            //         vk::PipelineBindPoint::GRAPHICS,
            //         self.pipeline_layout,
            //         0,
            //         &[self.descriptor_sets[self.base.current_frame]],
            //         &[self.ubo_alignment as u32 * i as u32],
            //     );

            //     self.base.device.cmd_bind_index_buffer(
            //         self.base.command_buffers[self.base.current_frame],
            //         mesh.index_buffer.buffer,
            //         0,
            //         vk::IndexType::UINT16,
            //     );
            //     self.base.device.cmd_draw_indexed(
            //         self.base.command_buffers[self.base.current_frame],
            //         mesh.index_count as u32,
            //         1,
            //         0,
            //         0,
            //         0,
            //     );
            // });

            self.base
                .device
                .cmd_end_render_pass(self.base.command_buffers[self.base.current_frame]);
            self.base
                .device
                .end_command_buffer(self.base.command_buffers[self.base.current_frame])
                .expect("Failed to record command buffer");
        };
    }

    #[inline]
    pub fn on_start(&mut self) {}

    #[inline]
    pub fn draw(&mut self) {
        unsafe {
            self.base
                .device
                .wait_for_fences(
                    &[self.base.next_frame[self.base.current_frame]],
                    true,
                    std::u64::MAX,
                )
                .unwrap();
            self.base
                .device
                .reset_fences(&[self.base.next_frame[self.base.current_frame]])
                .unwrap();

            let result = self.base.swapchain_loader.acquire_next_image(
                self.base.swapchain,
                std::u64::MAX,
                self.base.img_available[self.base.current_frame],
                vk::Fence::null(),
            );

            if let Err(vk::Result::ERROR_OUT_OF_DATE_KHR) = result {
                self.recreate_swapchain();
                return;
            }

            let (img_index, suboptimal) = result.unwrap();

            if suboptimal {
                self.recreate_swapchain();
            }

            self.base
                .device
                .reset_command_buffer(
                    self.base.command_buffers[self.base.current_frame],
                    vk::CommandBufferResetFlags::default(),
                )
                .unwrap();
            self.record_command_buffers(img_index as usize);

            let signal_semaphores = [self.base.render_finished[self.base.current_frame]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[self.base.img_available[self.base.current_frame]])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.base.command_buffers[self.base.current_frame]])
                .signal_semaphores(&signal_semaphores)
                .build();

            self.base
                .device
                .queue_submit(
                    self.base.queue,
                    &[submit_info],
                    self.base.next_frame[self.base.current_frame],
                )
                .expect("Failed to submit draw commands");

            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(std::slice::from_ref(&self.base.swapchain))
                .image_indices(std::slice::from_ref(&img_index));

            self.base
                .swapchain_loader
                .queue_present(self.base.queue, &present_info)
                .expect("Failed to present to screen");
        };

        self.base.current_frame = (self.base.current_frame + 1) % MAX_FRAME_DRAWS;
    }
}
