//! Data for clustering tests
pub mod clustered_2d;

#[cfg(test)]
/// Tests of the Point methods.
mod tests {
    #[allow(unused_imports)]
    use spectral::prelude::*;
    use std::include_str;
    use csv::{ ReaderBuilder };
    use crate::test_data::clustered_2d::Clustered2D;

    fn load_s2() -> &'static str {
        include_str!("test_s2.csv")
    }

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
}

