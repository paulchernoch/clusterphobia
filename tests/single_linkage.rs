#[allow(unused_imports)]
mod data;
use spectral::prelude::*;
// use crate::data::{load_s2, answer_key_2d, s2_points};
use crate::data::{s2_points};
use clusterphobia::clustering::single_linkage::SingleLinkage;

/// Test SingleLinkage::find_by_sorting against the s2 data set. 
#[test]
fn s2_single_linkage_find_by_sorting() {
    let mut points = s2_points();
    let finder = SingleLinkage::new(points.len() as u32, 20) // 20 bits because the largest coordinate value is just shy of one million
      .with_sort_distances_completely() // this forces use of find_by_sorting method
      .with_need_to_sort_by_hilbert_curve()
      .with_noise_skip_by(9);
    let linkage_result = finder.find(&mut points);
    let actual_distance = linkage_result.linkage_square_distance;

    // The linkage distance was derived by sorting the distances, plotting them on a graph,
    // and eyeballing it to see where the distances began to jump. 
    let expected_range = 3_100_000_000..3_500_000_000_u64;
    
    asserting(&format!("Actual Linkage square distance {} should be in expected range", actual_distance)).that(&expected_range.contains(&actual_distance)).is_equal_to(true);

    // Experiment shows that when things go well, the estimated number of clusters is 1.5x to 3x the true number.
    let is_cluster_count_reasonable = linkage_result.large_cluster_count >= 15 && linkage_result.large_cluster_count <= 45;

    asserting(&format!("Estimated number of clusters {}", linkage_result.large_cluster_count)).that(&is_cluster_count_reasonable).is_equal_to(true);
}
