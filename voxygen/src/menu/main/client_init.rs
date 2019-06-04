use client::{error::Error as ClientError, Client};
use common::comp;
use log::info;
use std::{
    net::ToSocketAddrs,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

#[cfg(feature = "discord")]
use crate::{discord, discord_instance};

#[derive(Debug)]
pub enum Error {
    // Error parsing input string or error resolving host name.
    BadAddress(std::io::Error),
    // Parsing yielded an empty iterator (specifically to_socket_addrs()).
    NoAddress,
    // Parsing/host name resolution successful but could not connect.
    ConnectionFailed(ClientError),
    ClientCrashed,
}

// Used to asynchronously parse the server address, resolve host names,
// and create the client (which involves establishing a connection to the server).
pub struct ClientInit {
    rx: Receiver<Result<Client, Error>>,
}
impl ClientInit {
    pub fn new(connection_args: (String, u16, bool), player: comp::Player, wait: bool) -> Self {
        let (server_address, default_port, prefer_ipv6) = connection_args;

        let (tx, rx) = channel();

        thread::spawn(move || {
            // Sleep the thread to wait for the single-player server to start up.
            if wait {
                info!("Waiting for server to come up...");
                thread::sleep(Duration::from_millis(500));
            }
            // Parse ip address or resolves hostname.
            // Note: if you use an ipv6 address, the number after the last colon will be used
            // as the port unless you use [] around the address.
            match server_address
                .to_socket_addrs()
                .or((server_address.as_ref(), default_port).to_socket_addrs())
            {
                Ok(socket_address) => {
                    let (first_addrs, second_addrs) =
                        socket_address.partition::<Vec<_>, _>(|a| a.is_ipv6() == prefer_ipv6);

                    let mut last_err = None;

                    for socket_addr in first_addrs.into_iter().chain(second_addrs) {
                        match Client::new(socket_addr, player.view_distance) {
                            Ok(mut client) => {
                                client.register(player);
                                let _ = tx.send(Ok(client));
                                
                                #[cfg(feature = "discord")]
                                {
                                    match discord_instance.lock() {
                                        Ok(mut disc) => {
                                            if !server_address.eq("127.0.0.1") {
                                                discord::send_singleplayer(&mut disc);
                                                disc.tx.send(discord::DiscordUpdate::Details(
                                                    server_address,
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("couldn't send Update to discord: {}", e)
                                        }
                                    }
                                }
                                
                                return;
                            }
                            Err(err) => {
                                match err {
                                    // Assume the connection failed and try next address.
                                    ClientError::Network(_) => {
                                        last_err = Some(Error::ConnectionFailed(err))
                                    }
                                    // TODO: Handle errors?
                                    _ => panic!(
                                        "Unexpected non-network error when creating client: {:?}",
                                        err
                                    ),
                                }
                            }
                        }
                    }
                    // Parsing/host name resolution successful but no connection succeeded.
                    let _ = tx.send(Err(last_err.unwrap_or(Error::NoAddress)));
                }
                Err(err) => {
                    // Error parsing input string or error resolving host name.
                    let _ = tx.send(Err(Error::BadAddress(err)));
                }
            }
        });

        ClientInit { rx }
    }
    /// Poll if the thread is complete.
    /// Returns None if the thread is still running, otherwise returns the Result of client creation.
    pub fn poll(&self) -> Option<Result<Client, Error>> {
        match self.rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(Error::ClientCrashed)),
        }
    }
}
