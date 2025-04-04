use snippets_multiplayer::TEST_SERVER_ADRESS;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .format_file(false)
        .format_line_number(true)
        .init();

    log::info!("Client attempts to connect to server {TEST_SERVER_ADRESS}...");

    let socket = tokio::net::TcpStream::connect(TEST_SERVER_ADRESS).await.unwrap();
    let client_address = socket.local_addr().unwrap();
    log::info!("Client {client_address} connected! Socket {socket:?}!");

}