

use std::collections::BinaryHeap;

use super::species::*;
use general::grid::*;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum Actor{
    FoodSpawn{time : usize},
    SporeSpawn{s : SpeciesID, p : Point, time : usize},
}
impl Actor{
    pub fn time(&self) -> usize{
        match self{
            Actor::FoodSpawn{time} => *time,
            Actor::SporeSpawn{time, ..} => *time,
        }
    }
}
impl PartialOrd for Actor{
    fn partial_cmp(&self, other : &Self) -> Option<std::cmp::Ordering>{
        Some(self.cmp(other))
    }
}
impl Ord for Actor{
    fn cmp(&self, other : &Self) -> std::cmp::Ordering{
        other.time().cmp(&self.time())
    }
}
pub trait CountableHeap{
    fn count_species(&self, s : SpeciesID) -> usize;
}
impl CountableHeap for BinaryHeap<Actor>{
    fn count_species(&self, s : SpeciesID) -> usize{
        self.clone().iter().filter(|actor|{
            match *actor{
                Actor::SporeSpawn{s : a_s, ..} => *a_s == s,
                _ => false
            }
        }).count()
    }
}