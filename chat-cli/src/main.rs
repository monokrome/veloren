use client::{Client, Event};
use common::{clock::Clock, comp};
use log::{error, info};
use std::time::Duration;

const FPS: u64 = 60;

fn main() {
    // Initialize logging.
    pretty_env_logger::init();

    info!("Starting chat-cli...");

    // Set up an fps clock.
    let mut clock = Clock::new();

    // Create a client.
    let mut client =
        Client::new(([127, 0, 0, 1], 59003), None).expect("Failed to create client instance");

    println!("Server info: {:?}", client.server_info);

    // TODO: Remove or move somewhere else, this doesn't work immediately after connecting
    println!("Ping: {:?}", client.get_ping_ms());

    println!("Players online: {:?}", client.get_players());

    client.register(comp::Player::new("test".to_string(), None));
    client.send_chat("Hello!".to_string());

    loop {
        let events = match client.tick(comp::Controller::default(), clock.get_last_delta()) {
            Ok(events) => events,
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        };

        for event in events {
            match event {
                Event::Chat(msg) => println!("[chat] {}", msg),
                Event::Disconnect => {} // TODO
            }
        }

        // Clean up the server after a tick.
        client.cleanup();

        // Wait for the next tick.
        clock.tick(Duration::from_millis(1000 / FPS));
    }
}
