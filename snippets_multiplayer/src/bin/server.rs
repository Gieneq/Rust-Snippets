use snippets_multiplayer::{game::common::Vector2F, multiplayer_server::MultiplayerServer, TEST_SERVER_ADRESS};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .format_file(false)
        .format_line_number(true)
        .init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let server = MultiplayerServer::bind(TEST_SERVER_ADRESS).await.unwrap();
        log::info!("MP-server, address:{:?}",  server.get_local_address().unwrap());
        
        let server_handler = server.run().await.unwrap();
    
        {
            let mut world = server_handler.world.lock().unwrap();
            world.create_entity_npc("Tuna", Vector2F::new(10.5, 20.3));
            // world.create_entity_npc("Starlette", Vector2F::new(-2.5, 0.0));
        }

        let (ctrlc_sender, ctrlc_receiver) = tokio::sync::oneshot::channel();
        let mut ctrlc_sender = Some(ctrlc_sender);

        ctrlc::set_handler(move || {
            log::info!("Captured ctrl-C, shutting down the server...");
            let sndr = ctrlc_sender.take().unwrap();
            sndr.send(()).unwrap();
        }).expect("Error setting Ctrl-C handler");

        ctrlc_receiver.await.unwrap();
        server_handler.shutdown().await.unwrap();
    })
}