use serde::{Serialize, Deserialize};
use hilbert::Point;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Clustered2D {
    pub id : usize,
    pub x : u32,
    pub y : u32,
    pub category : i32
}

impl From<&Clustered2D> for Point {
    fn from(point2d: &Clustered2D) -> Self {
        Point::new(point2d.id, &[point2d.x, point2d.y])
    }
}