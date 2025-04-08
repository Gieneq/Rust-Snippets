use snippets_multiplayer::{
    client_requests::ClientResponse, 
    game::common::Vector2F, 
    rendering::{
        renderer::State, 
        EntityView
    }, TEST_SERVER_ADRESS
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use std::{sync::{Arc, Mutex}, time::Duration};

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
            },
            WindowEvent::MouseWheel { 
                device_id: _, 
                delta, 
                phase: _ 
            } => {
                let scroll_sentivity: f32 = 0.1;
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        // y is +-1
                        self.scale *= (1.0 + scroll_sentivity).powf(y);
                    },
                    winit::event::MouseScrollDelta::PixelDelta(_physical_position) => todo!(),
                }
            },
            _ => (),
        }
    }
}

struct GuiClient {
    socket: tokio::net::TcpStream
}

struct GuiClientHandle {
    task_handle: tokio::task::JoinHandle<()>,
    entities: Arc<Mutex<Vec<EntityView>>>,
}

async fn client_do_request_await_response(
    req: &str,
    buf_reader: &mut tokio::io::BufReader<tokio::net::tcp::ReadHalf<'_>>,
    write: &mut tokio::net::tcp::WriteHalf<'_>,
) -> String {
    let mut buf_string = String::new();

    write.write_all(req.as_bytes()).await.unwrap();
    write.write_all(b"\n").await.unwrap();
    write.flush().await.unwrap();

    buf_reader.read_line(&mut buf_string).await.unwrap();
    buf_string.trim().to_string()
}

impl GuiClient {
    async fn connect<A: tokio::net::ToSocketAddrs + std::fmt::Debug>(addr: A) -> GuiClient {
        log::info!("Client attempts to connect to server {addr:?}...");

        let socket = tokio::net::TcpStream::connect(addr).await.unwrap();
        let client_address = socket.local_addr().unwrap();
        log::info!("Client {client_address} connected!");
        GuiClient {
            socket
        }
    }

    async fn run(mut self, shared_entities: Arc<Mutex<Vec<EntityView>>>) -> GuiClientHandle {
        let shared_entities_cloned = shared_entities.clone();

        let task_handle = tokio::task::spawn(async move {
            let (read_half, mut write_half) = self.socket.split();
            let mut buf_reader = tokio::io::BufReader::new(read_half);

            loop {
                let response = client_do_request_await_response(
                    "{\"type\":\"WorldCheck\"}",
                    &mut buf_reader,
                    &mut write_half
                ).await;
                log::trace!("Client got response '{response}'.");

                match serde_json::from_str(&response).unwrap() {
                    ClientResponse::WorldCheck { entities } => {
                        // Update shared data
                        if let Ok(mut entities_guard) = shared_entities.lock() {
                            entities_guard.clear();
                            for entiy in entities {
                                let color = if entiy.is_npc {
                                    [0.2, 0.2, 0.2]
                                } else {
                                    [0.3, 0.3, 0.3]
                                };

                                entities_guard.push(EntityView { 
                                    position: entiy.position, 
                                    size: entiy.size, 
                                    color
                                });
                                
                            }
                        }
                    },
                    _ => {},
                }

                tokio::time::sleep(Duration::from_millis(32)).await;
            }

        });

        GuiClientHandle {
            task_handle,
            entities: shared_entities_cloned
        }
    }
}

impl GuiClientHandle {
    async fn wait_until_finished(self) -> Result<(), tokio::task::JoinError> {
        self.task_handle.await
    }
}

#[tokio::main]
async fn main() {
    // wgpu uses `log` for all of our logging, so we initialize a logger with the `env_logger` crate.
    
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .format_timestamp_millis()
        .format_file(false)
        .format_line_number(true)
        .init();


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
        scale: 0.05,
        ..Default::default()
    };
    
    let client_handler = GuiClient::connect(TEST_SERVER_ADRESS).await
        .run(app.entities.clone()).await;

    event_loop.run_app(&mut app).unwrap();

    client_handler.wait_until_finished().await.unwrap();
}