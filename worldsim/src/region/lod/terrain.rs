use crate::lodstore::{
    LodData,
    LayerInfo,
    LodConfig,
    index::LodIndex,
};
use vek::*;

#[derive(Debug, Clone)]
pub struct Region9 {
    precent_air: f32,
    percent_forrest: f32,
    percent_lava: f32,
    percent_water: f32,
    child_id: Option<u32>, // Chunk5 2^(7*3), this is valid
}

#[derive(Debug, Clone)]
pub struct Chunk5 {
    precent_air: f32,
    percent_forrest: f32,
    percent_lava: f32,
    percent_water: f32,
    child_id: Option<u32>, // see Block0 2^(12*3)
}

#[derive(Debug, Clone)]
pub struct Block0 {
    material: u32,
    child_id: Option<u32>,// In reality 2^(16*3) SubBlock_4 should be possible, but 2^48 subblocks would kill anything anyway, so save 2 bytes here
}

#[derive(Debug, Clone)]
pub struct SubBlock_4 {
    material: u32,
}

impl LayerInfo for Region9 {
    fn get_child_index(self: &Self) -> Option<usize> {
        self.child_id.map(|n| n as usize)
    }
    const child_layer_id: Option<u8> = Some(9);
    const layer_volume: LodIndex = LodIndex {data: Vec3{x: 1, y: 1,z: 1}};
    const child_len: usize = 4096;//2_usize.pow(Self::child_dim*3);
}

impl LayerInfo for Chunk5 {
    fn get_child_index(self: &Self) -> Option<usize> {
        self.child_id.map(|n| n as usize)
    }
    const child_layer_id: Option<u8> = Some(4);
    const layer_volume: LodIndex = LodIndex {data: Vec3{x: 1, y: 1,z: 1}};
    const child_len: usize = 32768;//2_usize.pow(Self::child_dim*3);
}

impl LayerInfo for Block0 {
    fn get_child_index(self: &Self) -> Option<usize> {
        self.child_id.map(|n| n as usize)
    }
    const child_layer_id: Option<u8> = Some(0);
    const layer_volume: LodIndex = LodIndex {data: Vec3{x: 1, y: 1,z: 1}};
    const child_len: usize = 4096;//2_usize.pow(Self::child_dim*3);
}

impl LayerInfo for SubBlock_4 {
    fn get_child_index(self: &Self) -> Option<usize> {
        None
    }
    const child_layer_id: Option<u8> = None;
    const layer_volume: LodIndex = LodIndex {data: Vec3{x: 1, y: 1,z: 1}};
    const child_len: usize = 0;
}

#[derive(Debug, Clone)]
pub struct TerrainLodConfig {}

impl LodConfig for TerrainLodConfig {
    type L0 = SubBlock_4;
    type L1 = ();
    type L2 = ();
    type L3 = ();
    type L4 = Block0;
    type L5 = ();
    type L6 = ();
    type L7 = ();
    type L8 = ();
    type L9 = Chunk5;
    type L10 = ();
    type L11 = ();
    type L12 = ();
    type L13 = Region9;
    type L14 = ();
    type L15 = ();

    const anchor_layer_id: u8 = 13;

    fn setup(&mut self) {

    }
}

pub type Terrain = LodData<TerrainLodConfig>;



/*
impl Layer for Terrain {
    fn new() -> LodLayer<Terrain> {
        let mut n = LodLayer::<Terrain>::new_data(Terrain::Unused11);
        Self::drill_down(&mut n);
        n
    }

    fn get_level(layer: &LodLayer<Self>) -> i8 {
        match &layer.data {
            Terrain::Unused11 => LAYER5,
            Terrain::Region9{..} => LAYER4,
            Terrain::Chunk5{..} => LAYER3,
            Terrain::Block1{..} => LAYER2,
            Terrain::SubBlock_4{..} => -LAYER1,
        }
    }

    fn get_lower_level(layer: &LodLayer<Self>) -> Option<i8> {
        match &layer.data {
            Terrain::Unused11 => Some(LAYER4),
            Terrain::Region9{..} => Some(LAYER3),
            Terrain::Chunk5{..} => Some(LAYER2),
            Terrain::Block1{..} => Some(LAYER1),
            Terrain::SubBlock_4{..} => None,
        }
    }

    fn drill_down(layer: &mut  LodLayer<Terrain>) {
        match &layer.data {
            Terrain::Unused11 => {
                let n = LodLayer::new_data(Terrain::Region9{
                    precent_air: 1.0,
                    percent_forrest: 0.0,
                    percent_lava: 0.0,
                    percent_water: 0.0,
                });
                layer.childs = vec![n; 2_usize.pow((LAYER5-LAYER4) as u32 *3)];
            },
            Terrain::Region9{..} => {
                let n = LodLayer::new_data(Terrain::Chunk5{
                    precent_air: 1.0,
                    percent_forrest: 0.0,
                    percent_lava: 0.0,
                    percent_water: 0.0,
                });
                layer.childs = vec![n; 2_usize.pow((LAYER4-LAYER3) as u32 *3)];
            },
            Terrain::Chunk5{..} => {
                let n = LodLayer::new_data( Terrain::Block1{
                    material: 10,
                });
                layer.childs = vec![n; 2_usize.pow((LAYER3-LAYER2) as u32 *3)];
            },
            Terrain::Block1{..} => {
                let n = LodLayer::new_data( Terrain::SubBlock_4{
                    material: 10,
                });
                layer.childs = vec![n; 2_usize.pow((LAYER2-LAYER1) as u32 *3)];
            },
            Terrain::SubBlock_4{..} => {
                panic!("cannot drillDown further")
            },
        }
    }
    fn drill_up(parent: &mut LodLayer<Terrain>) {
        match &parent.data {
            Terrain::Unused11 => {
                panic!("cannot drillUp further")
            },
            Terrain::Region9{..} => {
                //recalculate values here
                parent.data = Terrain::Region9{
                    precent_air: 1.0,
                    percent_forrest: 0.0,
                    percent_lava: 0.0,
                    percent_water: 0.0,
                };
                parent.childs = vec![];
            },
            Terrain::Chunk5{..} => {
                parent.data = Terrain::Chunk5{
                    precent_air: 1.0,
                    percent_forrest: 0.0,
                    percent_lava: 0.0,
                    percent_water: 0.0,
                };
                parent.childs = vec![];
            },
            Terrain::Block1{..} => {
                parent.data = Terrain::Block1{
                    material: 10,
                };
                parent.childs = vec![];
            },
            Terrain::SubBlock_4{..} => {
                parent.data = Terrain::SubBlock_4{
                    material: 10,
                };
                parent.childs = vec![];
            },
        }
    }
}*/