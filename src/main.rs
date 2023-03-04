use renderer::{base::RendererBase, runtime::Renderer};
use winit::{
    dpi::LogicalSize,
    event::Event,
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{self, Window, WindowBuilder},
};

pub mod engine;
pub mod renderer;

fn main() {
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(800, 800))
        .with_resizable(false)
        .build(&event_loop)
        .expect("Failed to create window");

    let mut renderer = Renderer::new(&window);

    event_loop.run_return(move |event, _, control_flow| match event {
        Event::WindowEvent { event: e, .. } => match e {
            winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),

            _ => (),
        },
        Event::RedrawEventsCleared => renderer.draw(),
        _ => (),
    });
}
