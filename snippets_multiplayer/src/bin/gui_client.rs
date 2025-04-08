use snippets_multiplayer::{game::common::Vector2F, rendering::{renderer::State, EntityView}};

use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{
        ActiveEventLoop, 
        ControlFlow, 
        EventLoop
    },
    window::{
        Window, 
        WindowId
    },
};

#[derive(Default)]
struct App {
    state: Option<State>,
    entities: Arc<Mutex<Vec<EntityView>>>,
    scale: f32
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        let entities = self.entities.clone();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                state.render(entities, self.scale);
                // Emits a new redraw requested event.
                state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                state.resize(size);
            }
            _ => (),
        }
    }
}

fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    //
    // To change the log level, set the `RUST_LOG` environment variable. See the `env_logger`
    // documentation for more information.
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App {
        scale: 1.0,
        
        // for test purpose
        entities: Arc::new(Mutex::new(vec![
            EntityView {
                position: Vector2F::new(0.0, 0.0), 
                size: Vector2F::new(0.2, 0.2), 
                color: [0.2, 0.2, 0.2] 
            },
            EntityView {
                position: Vector2F::new(-0.5, 0.0), 
                size: Vector2F::new(0.3, 0.3), 
                color: [0.3, 0.3, 0.3] 
            },
        ])),
        ..Default::default()
    };

    event_loop.run_app(&mut app).unwrap();
}