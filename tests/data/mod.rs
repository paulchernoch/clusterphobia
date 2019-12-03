use std::include_str;
use csv::{ ReaderBuilder };
use hilbert::Point;
use self::clustered_2d::Clustered2D;

pub mod clustered_2d;

pub fn load_s2() -> &'static str {
    include_str!("test_s2.csv")
}
    
pub fn s2_points() -> Vec<Point> {
    let s2_data = load_s2();
    let mut rdr = ReaderBuilder::new()
        .from_reader(s2_data.as_bytes());
    let iter = rdr.deserialize::<Clustered2D>();
    let mut points = Vec::new();
    for record in iter {
        match record {
            Ok(point2d) => {
                points.push(point2d.into());
            },
            Err(err) => {  
                panic!("Unable to parse point. {:?}", err);
            }
        }
    }
    points
}
