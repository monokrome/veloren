use client::Client;
use common::clock::Clock;
use log::info;
use portpicker::pick_unused_port;
use server::{Event, Input, Server};
use std::{
    net::SocketAddr,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

const TPS: u64 = 30;

enum Msg {
    Stop,
}

/// Used to start and stop the background thread running the server
/// when in singleplayer mode.
pub struct Singleplayer {
    server_thread: JoinHandle<()>,
    sender: Sender<Msg>,
}

impl Singleplayer {
    pub fn new(client: Option<&Client>) -> (Self, SocketAddr) {
        let (sender, receiver) = channel();

        let sock = SocketAddr::from((
            [127, 0, 0, 1],
            pick_unused_port().expect("Failed to find unused port!"),
        ));

        // Create server
        let server = Server::bind(sock.clone()).expect("Failed to create server instance!");

        let server = match client {
            Some(client) => server.with_thread_pool(client.thread_pool().clone()),
            None => server,
        };

        let thread = thread::spawn(move || {
            run_server(server, receiver);
        });

        (
            Singleplayer {
                server_thread: thread,
                sender,
            },
            sock,
        )
    }
}

impl Drop for Singleplayer {
    fn drop(&mut self) {
        // Ignore the result
        let _ = self.sender.send(Msg::Stop);
    }
}

fn run_server(mut server: Server, rec: Receiver<Msg>) {
    info!("Starting server-cli...");

    // Set up an fps clock
    let mut clock = Clock::new();

    loop {
        let events = server
            .tick(Input::default(), clock.get_last_delta())
            .expect("Failed to tick server!");

        for event in events {
            match event {
                Event::ClientConnected { entity } => info!("Client connected!"),
                Event::ClientDisconnected { entity } => info!("Client disconnected!"),
                Event::Chat { entity, msg } => info!("[Client] {}", msg),
            }
        }

        // Clean up the server after a tick.
        server.cleanup();

        match rec.try_recv() {
            Ok(_msg) => break,
            Err(err) => match err {
                TryRecvError::Empty => (),
                TryRecvError::Disconnected => break,
            },
        }

        // Wait for the next tick.
        clock.tick(Duration::from_millis(1000 / TPS));
    }
}
