pub mod index;
use std::sync::Arc;
use std::collections::HashMap;
use vek::*;
use index::{
    LodIndex,
    length_to_index,
    two_pow_u,
    relative_to_1d,
};

/*
LOD Data contains different Entries in different vecs, every entry has a "pointer" to it's child start.
This is the structure to store a region and all subscribed information
*/
pub trait LayerInfo {
    fn get_child_index(self: &Self) -> Option<usize>;
    const layer_volume: LodIndex; // e.g. (1|1|1) for l0 or (4|4|4) for l2 optimization
    const child_layer_id: Option<u8>;
    const child_len: usize; //number of childs on this layer, MUST BE 2^(SELF::child_dim*3)
}

pub trait LodConfig {
    type L0: LayerInfo; // 2^-4
    type L1: LayerInfo;
    type L2: LayerInfo;
    type L3: LayerInfo;
    type L4: LayerInfo; // 2^0
    type L5: LayerInfo;
    type L6: LayerInfo;
    type L7: LayerInfo;
    type L8: LayerInfo;
    type L9: LayerInfo;
    type L10: LayerInfo;
    type L11: LayerInfo;
    type L12: LayerInfo;
    type L13: LayerInfo;
    type L14: LayerInfo;
    type L15: LayerInfo; // 2^11

    const anchor_layer_id: u8;

    fn setup(&mut self);
}

/*
There is another optimization problem: We have OWNED data and foreign DATA in the struct, but we don't want the foreign data to take a lot of space if unused
But both needs to be accessible transparent without overhead in calculation, difficult.
Imagine a Terrain, which top level is L13, so it would have 64 entries for the owned and 1664 for foreign data if everything is filled.
So we really only fill the boarder giving us 152 border areas.
One could think about multiple entry levels for foreign and owned data, but that means, that foreign data without a parent would exist, which might break algorithms....
So for now we go with a single anchorlevel for now, and hope the designer chooses good levels
*/

//ERROR NEXT STEP IS TO WORK ON SYSTEMS IN ORDER TO KNOW WHAT EXACTLY WE NEED. BUILD A FAKE "RASTERIZER" E:G:

#[derive(Debug, Clone)]
pub struct LodData<X: LodConfig> {
    pub layer0: Vec<X::L0>, // 1/16
    pub layer1: Vec<X::L1>, // 1/8
    pub layer2: Vec<X::L2>, // 1/4
    pub layer3: Vec<X::L3>, // 1/2
    pub layer4: Vec<X::L4>, // 1
    pub layer5: Vec<X::L5>, // 2
    pub layer6: Vec<X::L6>, // 4
    pub layer7: Vec<X::L7>, // 8
    pub layer8: Vec<X::L8>, // 16
    pub layer9: Vec<X::L9>, // 32
    pub layer10: Vec<X::L10>, // 64
    pub layer11: Vec<X::L11>, // 128
    pub layer12: Vec<X::L12>, // 256
    pub layer13: Vec<X::L13>, // 512
    pub layer14: Vec<X::L14>, // 1024
    pub layer15: Vec<X::L15>,  // 2048
    pub anchor: HashMap<LodIndex, usize>,
}

impl<X: LodConfig> LodData<X> {
    pub fn new() -> Self {
        Self {
            layer0: Vec::new(),
            layer1: Vec::new(),
            layer2: Vec::new(),
            layer3: Vec::new(),
            layer4: Vec::new(),
            layer5: Vec::new(),
            layer6: Vec::new(),
            layer7: Vec::new(),
            layer8: Vec::new(),
            layer9: Vec::new(),
            layer10: Vec::new(),
            layer11: Vec::new(),
            layer12: Vec::new(),
            layer13: Vec::new(),
            layer14: Vec::new(),
            layer15: Vec::new(),
            anchor: HashMap::new(),
        }
    }

    /*
    Da fuq is his code you might ask,
    but seriosly. because of logic reasons you have to know the level you want anyway, so we go for it ;)
    */

    fn int_get0<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L0 {
        debug_assert_eq!(T::child_layer_id, X::L0::child_layer_id);
        &self.layer0[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get1<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L1 {
        debug_assert_eq!(T::child_layer_id, X::L1::child_layer_id);
        &self.layer1[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get2<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L2 {
        debug_assert_eq!(T::child_layer_id, X::L2::child_layer_id);
        &self.layer2[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get3<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L3 {
        debug_assert_eq!(T::child_layer_id, X::L3::child_layer_id);
        &self.layer3[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get4<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L4 {
        debug_assert_eq!(T::child_layer_id, X::L4::child_layer_id);
        &self.layer4[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get5<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L5 {
        debug_assert_eq!(T::child_layer_id, X::L5::child_layer_id);
        &self.layer5[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get6<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L6 {
        debug_assert_eq!(T::child_layer_id, X::L6::child_layer_id);
        &self.layer6[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get7<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L7 {
        debug_assert_eq!(T::child_layer_id, X::L7::child_layer_id);
        &self.layer7[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get8<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L8 {
        debug_assert_eq!(T::child_layer_id, X::L8::child_layer_id);
        &self.layer8[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get9<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L9 {
        debug_assert_eq!(T::child_layer_id, X::L9::child_layer_id);
        &self.layer9[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get10<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L10 {
        debug_assert_eq!(T::child_layer_id, X::L10::child_layer_id);
        &self.layer10[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get11<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L11 {
        debug_assert_eq!(T::child_layer_id, X::L11::child_layer_id);
        &self.layer11[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get12<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L12 {
        debug_assert_eq!(T::child_layer_id, X::L12::child_layer_id);
        &self.layer12[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get13<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L13 {
        debug_assert_eq!(T::child_layer_id, X::L13::child_layer_id);
        &self.layer13[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get14<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L14 {
        debug_assert_eq!(T::child_layer_id, X::L14::child_layer_id);
        &self.layer14[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    fn int_get15<T: LayerInfo>(&self, parent: &T, index: LodIndex, parent_index: LodIndex) -> &X::L15 {
        debug_assert_eq!(T::child_layer_id, X::L15::child_layer_id);
        &self.layer15[relative_to_1d(index - parent_index, T::layer_volume)]
    }

    pub fn get15(&self, index: LodIndex) -> &X::L15 {
        debug_assert!(Some(X::anchor_layer_id) > X::L15::child_layer_id);
        &self.layer15[0]
    }



    /*
        pub fn int_get_lower(&self, index: LodIndex) -> &dyn LayerInfo {
            let y= match X::anchor_layer_id {
                0 => &self.layer0[1],
                15 => {
                    let x = self.anchor[&index];
                    let t15 = self.layer15[x];
                    match X::L15::child_layer_id {
                        Some(14) => {},
                    }
                },
                _ => &self.layer0[2],
            };
            &self.layer0[0]
        }

        pub fn get0(&self, index: LodIndex) -> &X::L0 {
            let y= match X::anchor_layer_id {
                0 => &self.layer0[1],
                15 => {
                    let x = self.anchor[&index];
                    let t15 = self.layer15[x];
                    match X::L15::child_layer_id {
                        Some(14) => {},
                    }
                },
                _ => &self.layer0[2],
            };
            &self.layer0[0]
        }
    */
    pub fn get1(&self, index: LodIndex) -> &X::L1 {
        &self.layer1[0]
    }





    // option 1, but cannot specify when to stop ...
    /*fn query(&mut self) -> (Vec<X::L0>, Vec<X::L1>) {

    }*/
}

impl LayerInfo for () {
    fn get_child_index(self: &Self) -> Option<usize> {
        None
    }
    const child_layer_id: Option<u8> = Some(0);
    const layer_volume: LodIndex = LodIndex {data: Vec3{x: 1, y: 1,z: 1}};
    const child_len: usize = 0;
}

/*
#[derive(Debug, Clone)]
pub struct LodLayer<E> {
    pub data: E,
    pub childs: Vec<LodLayer<E>>, //Optimization potential: size_of<Vec> == 24 and last layer doesnt need Vec at all.
}

pub trait Layer: Sized {
    fn new() -> LodLayer<Self>;

    fn get_level(layer: &LodLayer<Self>) -> i8;
    fn get_lower_level(layer: &LodLayer<Self>) -> Option<i8>;

    /*Drills down the layer and creates childs*/
    fn drill_down(layer: &mut  LodLayer<Self>);

    /*needs to recalc parent values and remove childs*/
    fn drill_up(parent: &mut LodLayer<Self>);
}

impl<E> LodLayer<E> {
    pub fn new_data(data: E) -> Self {
        Self {
            data,
            childs: Vec::new(),
        }
    }
}

impl<E: Layer> LodLayer<E> {
    // gets the internal index on this layer from relative position

    fn get_internal_index(&self, relative: LodIndex) -> Vec3<u16> {
        let ll = length_to_index(E::get_lower_level(self).expect("your type is wrong configured!, configure Layer trait correctly"));
        let length_per_children: u16 = two_pow_u(ll);
        let child_index = relative.map(|i| (i / length_per_children));
        return child_index;
    }

    fn get_internal_index_and_remainder(&self, relative: LodIndex) -> (Vec3<u16>, LodIndex) {
        let ll = length_to_index(E::get_lower_level(self).expect("your type is wrong configured!, configure Layer trait correctly"));
        let length_per_children: u16 = two_pow_u(ll);
        let child_index = relative.map(|i| (i / length_per_children));
        let remainder_index = relative.map2(child_index, |i,c| (i - c * length_per_children));
        return (child_index, remainder_index);
    }

    /*flatten the (1,2,3) child to 1*4+2*4+3*3*4 = 48*/
    fn get_flat_index(&self, internal_index: Vec3<u16>) -> usize {
        let ll = E::get_lower_level(self).expect("your type is wrong configured!, configure Layer trait correctly");
        let cl = E::get_level(self);
        let childs_per_dimentsion = (cl - ll) as usize;
        let index = internal_index.x as usize + internal_index.y as usize * childs_per_dimentsion + internal_index.z as usize * childs_per_dimentsion * childs_per_dimentsion;
        return index;
    }

    //index must be local to self
    fn get(&self, relative: LodIndex) -> &LodLayer<E> {
        // index is local for now
        if self.childs.is_empty() {
            return &self
        } else {
            let (int, rem) = self.get_internal_index_and_remainder(relative);
            let index = self.get_flat_index(int);
            &self.childs.get(index).unwrap().get(rem)
        }
    }

    /*
    These functions allow you to make the LodLayer provide a certain LOD for the specified area
    */
    /*is at least minimum or maximum*/
    pub fn make_at_least(&mut self, lower: LodIndex, upper: LodIndex, level: i8) {
        if E::get_level(self) <= level {
            //println!("done");
            return;
        }
        let (li, lr) = self.get_internal_index_and_remainder(lower);
        let (ui, ur) = self.get_internal_index_and_remainder(upper);

        if self.childs.is_empty() {
            E::drill_down(self);
            //println!("dd");
        }
        let ll = length_to_index(E::get_lower_level(self).expect("your type is wrong configured!, configure Layer trait correctly"));
        let length_per_children: u16 = two_pow_u(ll);
        //println!("li {} lr {} ui {} ur {} length {}", li, lr, ui, ur, length_per_children);
        if E::get_lower_level(self).unwrap() <= level {
            //println!("done");
            return;
        }

        for z in li.z..ui.z+1 {
            for y in li.y..ui.y+1 {
                for x in li.x..ui.x+1 {
                    //recursive call make_at_least
                    let mut new_lower = LodIndex::new(0,0,0);
                    let mut new_upper = LodIndex::new(length_per_children-1,length_per_children-1,length_per_children-1);
                    if x == li.x { new_lower.x = lr.x; }
                    if y == li.y { new_lower.y = lr.y; }
                    if z == li.z { new_lower.z = lr.z; }
                    if x == ui.x { new_upper.x = ur.x; }
                    if y == ui.y { new_upper.y = ur.y; }
                    if z == ui.z { new_upper.z = ur.z; }
                    let index = self.get_flat_index(LodIndex::new(x,y,z));
                    //println!("lo {} up {} layer {} inr {} in {}, vec {}", new_lower, new_upper, E::get_level(self), LodIndex::new(x,y,z), index, self.childs.len());
                    self.childs[index].make_at_least( new_lower, new_upper, level );

                }
            }
        }
    }
    fn make_at_most(&mut self, lower: LodIndex, upper: LodIndex, level: i8) {

    }
    fn make_exactly(&mut self, lower: LodIndex, upper: LodIndex, level: i8) {

    }
}
*/
