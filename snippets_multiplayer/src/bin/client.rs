use snippets_multiplayer::TEST_SERVER_ADRESS;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .format_file(false)
        .format_line_number(true)
        .init();

    log::info!("Client attempts to connect to server {TEST_SERVER_ADRESS}...");

    let mut socket = tokio::net::TcpStream::connect(TEST_SERVER_ADRESS).await.unwrap();
    let client_address = socket.local_addr().unwrap();
    log::info!("Client {client_address} connected! Socket {socket:?}!");

    let (read, mut write) = socket.split();

    let mut buf_reader = tokio::io::BufReader::new(read);

    write.write_all(b"Hello from client\n").await.unwrap();
    write.flush().await.unwrap();
    let mut buf = String::new();

    buf_reader.read_line(&mut buf).await.unwrap();
    println!("Got '{buf}'");
}