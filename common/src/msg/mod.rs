pub mod chat;
pub mod client;
pub mod ecs_packet;
pub mod server;

// Reexports
pub use self::client::ClientMsg;
pub use self::ecs_packet::{EcsCompPacket, EcsResPacket};
pub use self::server::{RequestStateError, ServerInfo, ServerMsg};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClientState {
    Pending,
    Connected,
    Registered,
    Spectator,
    Dead,
    Character,
}
