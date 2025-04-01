use std::{
    net::SocketAddr, 
    time::Duration
};

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

mod echo_server {
    type EchoHook = dyn Fn(String, String) + 'static + Send + Sync;

    use std::{sync::Arc, time::Duration};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    #[derive(Debug, thiserror::Error)]
    pub enum EchoServerError {
        #[error("IoError, reason='{0}'")]
        IoError(#[from] tokio::io::Error),
        
        #[error("TaskJoinError, reason='{0}'")]
        TaskJoinError(#[from] tokio::task::JoinError),

        #[error("KillFailed")]
        KillFailed,
    }
    
    pub struct EchoServer {
        listener: tokio::net::TcpListener,
        queue_capacity: usize,
        msg_handler: Option<Arc::<dyn Fn(String, String) + Send + Sync>>,
    }

    pub struct EchoServerHandler {
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
        task_handler: tokio::task::JoinHandle<()>,
        msg_rx: tokio::sync::mpsc::Receiver<String>, // no longer shutdown at drop
    }
    
    impl EchoServer {
        /// Bind listener to address ready to be started
        pub async fn bind<A: tokio::net::ToSocketAddrs>(addr: A) -> Result<Self, EchoServerError> {
            Ok(Self {
                listener: tokio::net::TcpListener::bind(addr).await?,
                queue_capacity: 32,
                msg_handler: None
            })
        }

        pub fn with_listener<F: Fn(String, String) + 'static + Send + Sync>(mut self, msg_handler: F) -> Self {
            self.msg_handler = Some(Arc::new(msg_handler));
            self
        }

        pub async fn bind_any_local() -> Result<Self, EchoServerError> {
            Self::bind("127.0.0.1:0").await
        }

        pub fn get_local_address(&self) -> std::io::Result<std::net::SocketAddr> {
            self.listener.local_addr()
        }

        /// Run listening in background
        #[must_use = "EchoServerHandler must be stored to keep the server alive"]
        pub fn run(self) -> Result<EchoServerHandler, EchoServerError> {
            /// Helper function to process messages in connections
            async fn handle_connection(
                mut stream: tokio::net::TcpStream, 
                client_addr: std::net::SocketAddr, 
                msg_tx: tokio::sync::mpsc::Sender<String>,
                handler: Option<Arc<EchoHook>>
            ) {
                println!("Incomming connection {client_addr:?}");
                let (reader, mut writer) = stream.split();

                let mut read_buffer = tokio::io::BufReader::new(reader);
                let mut line_buf = String::new();

                loop {
                    match read_buffer.read_line(&mut line_buf).await {
                        Ok(0) => {
                            println!("Client {client_addr:?} closed connection");
                            break;
                        },
                        Ok(_) => {
                            if let Err(e) = msg_tx.try_send(line_buf.clone()) {
                                println!("Couldnt queue messages from {client_addr:?} reason {e}");
                            }

                            if let Some(h) = handler.as_ref() {
                                h(client_addr.to_string(), line_buf.clone());
                            }

                            if let Err(e) = writer.write_all(line_buf.as_bytes()).await {
                                println!("Couldnt write back to client {client_addr:?} reason {e}");
                            }
                            writer.flush().await.unwrap();
                            line_buf.clear();
                        },
                        Err(e) => {
                            println!("Reading message from client {client_addr:?} failed, reason {e}");
                            break;
                        }
                    }
                }
            }

            let address = self.get_local_address()?;
            println!("Started echo server at {address}");

            let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

            let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(self.queue_capacity);
            // Holding msg_tx will prevent closing, dropping handler wont help

            let msg_handler = self.msg_handler.clone();

            // Spawn task to monitor incommingconenctions in background
            let task_handler = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = &mut shutdown_rx => {
                            // This signal will be captured despite 
                            // other branchin probress and cancel the other branch.
                            println!("Got shutdown signal");
                            break;
                        },
                        incomming_connection = self.listener.accept() => {
                            if let Ok((socket, address)) = incomming_connection {
                                let tx = msg_tx.clone();
                                let handler = msg_handler.clone();
                                tokio::spawn(async move {
                                    handle_connection(socket, address, tx, handler).await;
                                });
                            } else {
                                println!("Incomming connection error");
                            }

                            // Here drop connection disconnetes client
                        },
                    }
                }
            });

            Ok(EchoServerHandler {
                shutdown_tx,
                task_handler,
                msg_rx
            })
        }
    }

    impl EchoServerHandler {
        pub async fn shutdown(self) -> Result<(), EchoServerError> {
            println!("Shutting down server...");
            self.shutdown_tx.send(()).map_err(|_| EchoServerError::KillFailed)?;

            match self.task_handler.await {
                Ok(_) => {
                    println!("Server shutdown sucessfully!");
                    Ok(())
                },
                Err(_) => {
                    eprintln!("Shutting down server failed!");
                    Err(EchoServerError::KillFailed)
                },
            }
        }

        pub async fn await_incomming_msg(&mut self, duration: Option<Duration>) -> Result<Option<String>, tokio::time::error::Elapsed> {
            if let Some(timeout_duration) = duration {
                tokio::time::timeout(timeout_duration, self.msg_rx.recv()).await
            } else {
                Ok(self.msg_rx.recv().await)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[tokio::test]
        async fn test_shutdown() {
            let echo_server = EchoServer::bind_any_local().await.unwrap();
            let echo_serer_handler = echo_server.run().unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            echo_serer_handler.shutdown().await.unwrap();
        }

        async fn client_make_requests<A: tokio::net::ToSocketAddrs>(server_address: A, requests: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
            let mut client_socket = tokio::net::TcpStream::connect(server_address).await?;
            let (reader, mut writer) = client_socket.split();

            let mut read_buffer = tokio::io::BufReader::new(reader);
            let mut response_buffer = String::new();

            for &request in requests {
                writer.write_all(request.as_bytes()).await?;
                writer.flush().await?;

                tokio::time::sleep(Duration::from_millis(10)).await;
    
                tokio::time::timeout(Duration::from_millis(500), read_buffer.read_line(&mut response_buffer)).await.unwrap().unwrap();

                // println!("request={request}, response={response_buffer}");
    
                assert_eq!(request, response_buffer.as_str());
                response_buffer.clear();
            }

            writer.shutdown().await?;
            Ok(())
        }

        #[tokio::test]
        async fn test_echo_single_message() {
            let echo_server = EchoServer::bind_any_local().await.unwrap();
            let server_address = echo_server.get_local_address().unwrap();
            let echo_server_handle = echo_server.run().unwrap();

            client_make_requests(server_address, &["message\n"]).await.unwrap();
            echo_server_handle.shutdown().await.unwrap();
        }

        #[tokio::test]
        async fn test_echo_multiple_messages() {
            let echo_server = EchoServer::bind_any_local().await.unwrap();
            let server_address = echo_server.get_local_address().unwrap();
            let echo_server_handle = echo_server.run().unwrap();

            client_make_requests(server_address, &["message\n", "aaa\n", "hello1234$%\n"]).await.unwrap();
            echo_server_handle.shutdown().await.unwrap();
        }

        #[tokio::test]
        async fn test_echo_multiple_messages_with_queue() {
            let echo_server = EchoServer::bind_any_local().await.unwrap();
            let server_address = echo_server.get_local_address().unwrap();
            let mut echo_server_handle = echo_server.run().unwrap();

            client_make_requests(server_address, &["message\n", "1234\n"]).await.unwrap();

            let msg1 = echo_server_handle.await_incomming_msg(None).await.unwrap().unwrap();
            assert_eq!(msg1, "message\n");

            let msg2 = echo_server_handle.await_incomming_msg(None).await.unwrap().unwrap();
            assert_eq!(msg2, "1234\n");

            assert!(echo_server_handle.await_incomming_msg(Some(Duration::from_millis(100))).await.is_err());
            echo_server_handle.shutdown().await.unwrap();
        }

        #[tokio::test]
        async fn test_echo_multiple_clients_multiple_messages() {
            let clients_messages = [
                vec!["Hello world!\n"],
                vec!["Hello\n", "World\n"],
                vec!["Foo\n", "Bar\n", "Buzz\n", "Donk\n", "Gotit\n"],
                vec!["1234\n", "#$%^\n", "pddlaaass654sdjnt bksdf\n", "#34\n", "#7777777^\n", "asdASDasghj\n"],
                vec!["Foo\n"; 103],
            ];

            let echo_server = EchoServer::bind_any_local().await.unwrap();
            let server_address = echo_server.get_local_address().unwrap();
            let echo_server_handle = echo_server.run().unwrap();

            let task_handles = clients_messages
                .into_iter()
                .map(|msg| {
                    tokio::spawn(async move {
                        client_make_requests(server_address, &msg).await.unwrap();
                    })
                })
                .collect::<Vec<_>>();

            for handle in task_handles {
                handle.await.unwrap();
            }

            // not reaching 
            echo_server_handle.shutdown().await.unwrap();
        }
        

        #[tokio::test]
        async fn test_echo_multiple_messages_with_hook() {
            let echo_server = EchoServer::bind_any_local().await
                .unwrap()
                .with_listener(|a, b| {
                    println!(">> {a}: {b}");
                });
            let server_address = echo_server.get_local_address().unwrap();
            let echo_server_handle = echo_server.run().unwrap();

            client_make_requests(server_address, &["message\n", "aaa\n", "hello1234$%\n"]).await.unwrap();
            echo_server_handle.shutdown().await.unwrap();
        }
    }
}

mod echo_client {
    struct EchoClient {

    }
}

// fn main() {
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(async {
//         let echo_server = echo_server::EchoServer::bind("127.0.0.1:0").await.unwrap();
//         let server_address = echo_server.get_local_address().unwrap();
        
//         let _echo_server_handle = echo_server.run().unwrap();

//         println!("Working...");
//         tokio::time::sleep(Duration::from_secs(1)).await;
        
//         // echo_server_handle.shutdown().await.unwrap();

//         let mut client_socket = tokio::net::TcpStream::connect(server_address).await.unwrap();
//         let (reader, mut writer) = client_socket.split();

//         let mut read_buffer = tokio::io::BufReader::new(reader);
//         let request = "message\n";
//         let mut response_buffer = String::new();

//         writer.write_all(request.as_bytes()).await.unwrap();
//         writer.flush().await.unwrap();

//         read_buffer.read_line(&mut response_buffer).await.unwrap();
//         println!("Client got echo: '{response_buffer}'");
//         writer.shutdown().await.unwrap();

//         assert_eq!(request, response_buffer.as_str());
//     });
// }






