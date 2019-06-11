use client::{Client, Event};
use common::{clock::Clock, comp};
use log::{error, info};
use std::time::Duration;
use std::io::{self, Read};

const FPS: u64 = 60;

fn read_user_input() -> String {
    let mut buffer: String = String::new();

    io::stdin().read_line(&mut buffer).ok().expect("Failed to set a new username");

    buffer
}

fn main() {
    // Initialize logging.
    pretty_env_logger::init();

    info!("Starting chat-cli...");

    // Set up an fps clock.
    let mut clock = Clock::new();

    // Input buffer
    let mut input = String::new();

    // Initialize the client
    println!("Choose your username");
    let mut name = read_user_input();

    // Create a client.
    let mut client =
        Client::new(([127, 0, 0, 1], 59003), None).expect("Failed to create client instance");

    println!("Server info: {:?}", client.server_info);

    println!("Players online: {:?}", client.get_players());

    client.register(comp::Player::new(name, None));

    loop {
        let chat_msg = read_user_input();
        client.send_chat(chat_msg);

        let events = match client.tick(comp::Control::default(), clock.get_last_delta()) {
            Ok(events) => events,
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        };

        for event in events {
            match event {
                Event::Chat(msg) => println!("{}", msg),
                Event::Disconnect => {} // TODO
            }
        }

        // Clean up the server after a tick.
        client.cleanup();

        // Wait for the next tick.
        clock.tick(Duration::from_millis(1000 / FPS));
    }
}
