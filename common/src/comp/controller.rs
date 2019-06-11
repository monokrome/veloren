use specs::{Component, FlaggedStorage, VecStorage};
use vek::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Controller {
    pub move_dir: Vec2<f32>,
    pub jump: bool,
    pub glide: bool,
    pub attack: bool,
    pub respawn: bool,
}

impl Component for Controller {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}
