use specs::{Component, FlaggedStorage, NullStorage, VecStorage};
use vek::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Respawning;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Attacking {
    pub time: f32,
    pub applied: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Jumping;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Gliding;

impl Component for Respawning {
    type Storage = NullStorage<Self>;
}

impl Attacking {
    pub fn start() -> Self {
        Self {
            time: 0.0,
            applied: false,
        }
    }
}

impl Component for Attacking {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Component for Jumping {
    type Storage = NullStorage<Self>;
}

impl Component for Gliding {
    type Storage = NullStorage<Self>;
}
