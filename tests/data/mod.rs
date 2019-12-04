//! Data for clustering tests
//! 
//!   - test_s2.csv

use std::include_str;
use std::collections::HashMap;
use std::ops::Range;
use csv::{ ReaderBuilder };
use hilbert::Point;
use self::clustered_2d::Clustered2D;
use clusterphobia::clustering::{Clustering, integer_clustering};

pub mod clustered_2d;

pub fn load_s1() -> &'static str {
    include_str!("test_s1.csv")
}

/// Load the data file test_s2.csv into a text string.
/// 
/// The file has an id, x and y coordinates, and the gold key (ideal) categorization.
/// There are fifteen clusters that partially overlap, making it a moderately challenging
/// test of a clustering algorithm in low-dimensions.
/// This test file is built from data from: http://cs.joensuu.fi/sipu/datasets/
/// 
/// The data originally appeared in this paper:
/// 
/// ```
///    P. Fränti and S. Sieranoja
///    "K-means properties on six clustering benchmark datasets"
///    Applied Intelligence, 48 (12), 4743-4759, December 2018
///    https://doi.org/10.1007/s10489-018-1238-7
///    BibTex
/// 
/// ```
pub fn load_s2() -> &'static str {
    include_str!("test_s2.csv")
}

fn make_2d_points(csv_data : &str) -> Vec<Point> {
    let mut rdr = ReaderBuilder::new()
        .from_reader(csv_data.as_bytes());
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

pub fn s1_points() -> Vec<Point> {
    make_2d_points(load_s1())
}

pub fn s2_points() -> Vec<Point> {
    make_2d_points(load_s2())
}

/// Parse the CSV data for 2-dimensional points that come with an answer key
/// and cluster their ids according to the supplied categories.
#[allow(dead_code)]
pub fn answer_key_2d(categorized_csv_data : &'static str) -> Clustering<usize,usize,Range<usize>> {
    // Keep track of the id of the first point we find for each category.
    // We will use that as a seed around which to form the proper clusters.
    let mut category_to_first_point : HashMap<usize,usize> = HashMap::new();
    let mut clustering = integer_clustering();

    let mut rdr = ReaderBuilder::new()
        .from_reader(categorized_csv_data.as_bytes());
    let iter = rdr.deserialize::<Clustered2D>();
    for record in iter {
        match record {
            Ok(point2d) => {
                let Clustered2D { id, category, .. } = point2d;
                let first_point_opt = category_to_first_point.get(&(category as usize));
                match first_point_opt {
                    Some(seed_point_id) => {
                        clustering.merge(*seed_point_id, id);
                    },
                    None => {
                        // First point we have see for this category.
                        category_to_first_point.insert(category as usize, id);
                        let _ = clustering.add_to_new_cluster(id);
                    }
                }
            },
            Err(err) => {  
                panic!("Unable to parse point. {:?}", err);
            }
        }
    }
    clustering
}
