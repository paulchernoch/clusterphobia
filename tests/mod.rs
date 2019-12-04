//! Integration Tests
pub mod data;


#[allow(unused_imports)]
use spectral::prelude::*;
use csv::{ ReaderBuilder };
use crate::data::{load_s2, answer_key_2d};
use crate::data::clustered_2d::Clustered2D;
use clusterphobia::clustering::bcubed::BCubed;


/// Verify that we can read the CSV file and parse it into Clustered2D values. 
#[test]
fn parse_s2() {
    let s2_data = load_s2();
    let mut rdr = ReaderBuilder::new()
        .from_reader(s2_data.as_bytes());
    let iter = rdr.deserialize::<Clustered2D>();
    for (expected_id, record) in iter.enumerate() {
        match record {
            Ok(point2d) => {
                asserting(&format!("Point {} has wrong id", expected_id)).that(&point2d.id).is_equal_to(expected_id);
                let good_category = point2d.category >=1 && point2d.category <= 15;
                asserting(&format!("Point {} has out of range category {}", expected_id, point2d.category)).that(&good_category).is_equal_to(true);
            },
            Err(err) => {  
                panic!("Unable to parse point {}. {:?}", expected_id, err);
            }
        }
    }
}

/// Show that we can load a test data set (s2) with perfect categories into a Clustering, then compare 
/// it to another perfect Clustering and one with an error and get good results. 
#[test]
fn s2_answer_key() {
    let mut solution = answer_key_2d(load_s2());
    asserting("Expecting 15 clusters for s2").that(&solution.cluster_count()).is_equal_to(15);

    // If the clusterings are identical, the similarity should be one. 
    let gold = answer_key_2d(load_s2());
    let mut comparison = BCubed::compare(&solution, &gold, 0.5);
    asserting("Expect perfect similarity value of 1.0").that(&comparison.similarity()).is_equal_to(1.0);

    // If we merge two clusters and then compare again, homogeneity should suffer, but not completeness.
    solution.merge(1, 1000);
    comparison = BCubed::compare(&solution, &gold, 0.5);
    asserting("Expect a similarity value < 1.0").that(&(comparison.similarity() < 1.0)).is_equal_to(true);
    asserting(&format!("Expect perfect completeness (aka recall) value of 1.0, got {:?}", comparison)).that(&comparison.get_recall()).is_equal_to(1.0);
    asserting("Expect homogeneity (aka precision) value < 1.0").that(&(comparison.get_precision() < 1.0)).is_equal_to(true);
}


