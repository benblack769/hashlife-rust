
use std::cmp::Ordering;

#[derive(Copy,Clone,Hash,PartialEq,Eq)]
pub struct Point{
    pub x:i64,
    pub y:i64,
}

impl Ord for Point{
    fn cmp(&self, other: &Self) -> Ordering{
        match self.y.cmp(&other.y){
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.x.cmp(&other.x)
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

