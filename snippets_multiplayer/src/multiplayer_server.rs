use std::{sync::{Arc, Mutex}, time::Duration};

use crate::game::world::World;

#[derive(Debug, thiserror::Error)]
pub enum MultiplayerServerError {
    #[error("IoError, reason='{0}'")]
    IoError(#[from] tokio::io::Error),

    #[error("Failed to shutdown server")]
    ShutdownError,

    #[error("Could not join task, reason='{0}'")]
    TaskJoinError(#[from] tokio::task::JoinError),
}

pub struct MultiplayerServerHandler {
    world: Arc<Mutex<World>>,
    main_task_handler: tokio::task::JoinHandle<()>,
    shutdown_sender: tokio::sync::oneshot::Sender<()>,
}

pub struct MultiplayerServer {
    listener: tokio::net::TcpListener,
}

impl MultiplayerServer {
    const MAIN_LOOP_INTERVAL: Duration = Duration::from_millis(250); // Slow for testing purpose

    pub async fn bind_any_local() -> Result<Self, MultiplayerServerError> {
        Self::bind("127.0.0.1:0").await
    }

    pub async fn bind<A: tokio::net::ToSocketAddrs>(addr: A) -> Result<Self, MultiplayerServerError> {
        Ok(Self {
            listener: tokio::net::TcpListener::bind(addr).await?,
        })
    }

    pub fn get_local_address(&self) -> Result<std::net::SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }

    pub async fn run(self) -> Result<MultiplayerServerHandler, MultiplayerServerError> {
        let world = Arc::new(Mutex::new(World::new()));
        let world_shared = world.clone();

        let (shutdown_sender, mut shutdown_receiver) = tokio::sync::oneshot::channel();

        let main_task_handler = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut shutdown_receiver => {
                        // Received shutdown signal
                        println!("Received shut down signal...");
                        break;
                    },
                    _ = tokio::time::sleep(Self::MAIN_LOOP_INTERVAL) => {
                        // Execute every tick
                        if let Ok(mut world_lock) = world_shared.lock() {
                            world_lock.tick();
                        }
                    },
                }
            }
        });

        Ok(MultiplayerServerHandler {
            world,
            main_task_handler,
            shutdown_sender,
        })
    }
}

impl MultiplayerServerHandler {
    pub async fn shutdown(self) -> Result<(), MultiplayerServerError> {
        println!("Gracefully shutting down server...");
        self.shutdown_sender.send(()).map_err(|_| MultiplayerServerError::ShutdownError)?;
        self.main_task_handler.await?;
        println!("Server shut down successfully!");
        Ok(())
    }
}

#[tokio::test]
async fn test_server_creation() {
    let server = MultiplayerServer::bind_any_local().await.unwrap();
    let server_address = server.get_local_address().unwrap();
    println!("{server_address:?}");
    let server_handler = server.run().await.unwrap();
    tokio::time::sleep(Duration::from_millis(2000)).await;
    server_handler.shutdown().await.unwrap();
}