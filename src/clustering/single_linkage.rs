
use std::ops::Range;
use std::cmp::Ordering;
use std::cmp::{min,max};
use hilbert::Point;

// ........................... LinkageResult ..........................................

/// Result from computing the Linkage distance, including 
/// statistics about how many clusters are likely to be formed as a result
/// of clustering using that distance. 
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct LinkageResult {
    /// The square of the linkage distance. (Avoid unnecessary square roots.)
    /// 
    /// _This is the primary result value, not set until after `find` is called._
    pub linkage_square_distance : u64, 

    /// Counts how many pairwise point distances are larger than the `linkage_square_distance`.
    /// This is a crude estimate (on the high side) of how many clusters a rough clustering will
    /// yield using this `linkage_square_distance`.
    /// 
    /// _This a secondary result value, not set until after `find` is called._
    pub count_of_too_large_distances : u32,

    /// Counts how many runs of points sorted in Hilbert Curve order are composed of points 
    /// separated by no more than the `linkage_square_distance` 
    /// AND have a size that exceeds the `outlier_cluster_size`.
    /// This is an upper limit on the number of clusters that will result from 
    /// the full clustering algorithm. 
    /// 
    /// **Experiment shows that this tends to be from 1.5x to 3x the true number of clusters.**
    /// 
    /// _This a secondary result value, not set until after `find` is called._
    pub large_cluster_count : u32,

    /// Counts how many runs of points sorted in Hilbert Curve order are composed of points 
    /// separated by no more than the `linkage_square_distance` 
    /// AND have a size that is less than or equal to the `outlier_cluster_size`.
    /// 
    /// _This a secondary result value, not set until after `find` is called._
    pub outlier_cluster_count : u32,

    /// Counts how many individual points fall into clusters inferred to be outliers.
    /// 
    /// _This a secondary result value, not set until after `find` is called._
    pub outlier_count : u32
}

impl LinkageResult {
    pub fn new() -> Self {
        LinkageResult {
            linkage_square_distance : 0,
            count_of_too_large_distances : 0,
            large_cluster_count : 0,
            outlier_cluster_count : 0,
            outlier_count : 0
        }
    }
}

// ........................... SingleLinkage ..........................................


/// Given a set of points, finds the linkage distance to use when performing **single-link agglomerative clustering**.
/// 
/// When performing bottom-up agglomerative clustering of data, the decision to merge or not merge 
/// two points (or two clusters of points) into the same cluster is based on some distance measure.
/// The measure may be between two points (single-linkage) or between whole clusters 
/// (complete-linkage, Unweighted average linkage, Weighted average linkage, Centroid linkage, Minimum energy, others).
/// This class computes the **single-linkage distance**.
/// 
/// Two algorithms are provided:
/// 
///   - `find_by_sorting` - This is more accurate but slower, requiring a costly **O(N Log N) sort** of the distances between Points.
///   - `find_by_binning` - This is faster but less accurate, using a **linear O(N)** bucket sort with logarithmically-sized buckets to partially sort the data.
/// 
/// This analysis uses a heuristic. It attempts to duplicate what people do easily by eye: 
/// spot where the distribution of distances shows its first sudden jump, an elbow in the curve.
/// The physical insight is that pairs of points within the same cluster will be close together, pairs of points
/// from different clusters will be far apart, and there is a gray region of distances between them where there will be few pairs
/// of points.
/// 
/// When there is uncertainty between several values, we will prefer the smaller value.
/// In a bottom up algorithm, it is better to err on the side of failing to merge clusters that belong together
/// rather than merging clusters that do not belong together.
/// 
/// **THIS MEASUREMENT IS THE SINGLE MOST IMPORTANT CALCULATION TO GET RIGHT!!!**
/// 
///   - If the value is too high, **homogeneity** will suffer. (Points that do not belong together will be clustered together.)
///   - If the value is too low, **completeness** will suffer. (Points that belong together will be split apart.)
/// 
/// The number one cause of obtaining a bad value is high levels of noise in the data, which blurs the distinction between cluster boundaries. 
/// Other things to be avoided: 
///   - There may be a few duplicate points or really close points, then a gap to the typical range of distances. 
///     We must not falsely identify this first jump in distances as the linkage distance.
///   - If there is a great variety in the densities of clusters, then there could be additional gaps. 
/// 
/// While this algorithm does not actually cluster the points, it does estimate how many proper clusters and outlier clusters
/// would be generated from the derived linkage distance. 
/// 
/// The configuration of this class permits a fluent style. 
#[derive(Copy, Clone, Debug)]
pub struct SingleLinkage {
    /// Should the Hilbert transform be performed, use this number of bits per dimension to encode the coordinates. 
    bits_per_dimension : u8,

    /// If false, assume that the points to be analyzed have already been sorted in Hilbert Curve order. 
    /// If true, perform a sort by the Hilbert curve first.
    need_to_sort_by_hilbert_curve : bool,

    /// The linkage distance will be chosen so that at minimum, this number of clusters
    /// will be yielded by a rough clustering.
    /// The default is to use 1/(2√N). 
    /// 
    /// For example, if there are N = 5,000 points, yield a minimum of 35 clusters.
    /// Subsequent rounds of clustering often reduce this number by three, or 12 clusters. 
    minimum_cluster_count : u16,

    /// To accommodate noise, we will look for a sudden jump in distance not between adjacent points, but between 
    /// points separated by this number of positions (plus one) in sorted order. 
    /// The intent is to sharpen the signal. 
    /// 
    ///   - Too large a value will depress the resulting `linkage_square_distance`. 
    ///   - Too small a value will risk noise points throwing off the calculation, falsely increasing the `linkage_square_distance`.
    ///   - If in doubt, use 5. 
    ///   - A value of zero is permissible.
    noise_skip_by : u16,

    /// When counting potential clusters, do not count as a cluster any runs of consecutive points in Hilbert order whose 
    /// separation is less than the linkage_square_distance unless they exceed this size. 
    outlier_cluster_size : u16,

    /// If true, fully sort the distances as part of the analysis, which is more accurate but more expensive 
    /// (O(N Log N) instead of O(N)). 
    /// 
    /// If false, use a logarithmic bucket sort to partially sort the distances, 
    /// which is less accurate but also less expensive.
    sort_distances_completely : bool,

    /// It is not uncommon for a few very small distances to be followed by
    /// a proportionally huge increase, like going from 2 to 10 being a fivefold increase. 
    /// This threshold prevents upping the maximum_ratio seen until at least this many 
    /// distances have been checked. 
    /// 
    /// This defaults to ½N. 
    /// 
    ///   - Increase it if nearly half or more of your points are coincident or very close together. 
    ///   - Decrease it if you have a huge number of outliers (beyond 40% outliers). 
    lowest_index_for_checking_growth_ratio : u32 
}

impl SingleLinkage {
    /// Create a SingleLinkage with all values set to defaults.
    pub fn new(num_points : u32, bits_per_dimension : u8) -> Self {
        let mut minimum_cluster_count = (num_points as f64).sqrt() / 2.0;
        if minimum_cluster_count < 10.0 {
            minimum_cluster_count = 10.0;
        }
        SingleLinkage {
            bits_per_dimension,
            minimum_cluster_count : minimum_cluster_count as u16,
            need_to_sort_by_hilbert_curve : false,
            noise_skip_by : 5,
            outlier_cluster_size : 10,
            sort_distances_completely : true,
            lowest_index_for_checking_growth_ratio : num_points / 2
        }
    }

    /// Configure the algorithm to expect that the input points HAVE NOT yet been sorted
    /// in Hilbert Curve order, and so require that this sorting be performed by `find`.
    /// This sets `need_to_sort_by_hilbert_curve`.
    pub fn with_need_to_sort_by_hilbert_curve(mut self) -> Self {
        self.need_to_sort_by_hilbert_curve = true;
        self
    }

    /// Configure the algorithm to expect that the input points HAVE ALREADY been sorted
    /// in Hilbert Curve order, and so DO NOT require that this sorting be performed by `find`.
    /// This sets `need_to_sort_by_hilbert_curve`.
     pub fn without_need_to_sort_by_hilbert_curve(mut self) -> Self {
        self.need_to_sort_by_hilbert_curve = false;
        self
    }

    /// Configure the algorithm by setting a value for `noise_skip_by`. 
    pub fn with_noise_skip_by(mut self, noise_skip_by : u16) -> Self {
        self.noise_skip_by = noise_skip_by;
        self
    }

    /// Configure the algorithm by setting a value for `minimum_cluster_count`. 
    pub fn with_minimum_cluster_count(mut self, min_cluster_count : u16) -> Self {
        self.minimum_cluster_count = max(min_cluster_count, 6);
        self
    }

    /// Configure the algorithm by setting `sort_distances_completely` to true.
    /// This will provide the most accurate results at the expense of O(N Log N) running time.  
    pub fn with_sort_distances_completely(mut self) -> Self {
        self.sort_distances_completely = true;
        self
    }

    /// Configure the algorithm by setting `sort_distances_completely` to false.
    /// This will provide less accurate results with the benefit of a faster O(N) running time.  
    pub fn without_sort_distances_completely(mut self) -> Self {
        self.sort_distances_completely = false;
        self
    }

    /// Configure the algorithm by setting `lowest_index_for_checking_growth_ratio`.  
    pub fn with_lowest_index_for_checking_growth_ratio(mut self, index : u32) -> Self {
        self.lowest_index_for_checking_growth_ratio = index;
        self
    }

    /// Find the characteristic clustering distance, the linkage distance, along with some other useful results. 
    /// 
    ///   - points - Points to analyze. As a side effect, these points will be sorted in Hilbert Curve order
    ///     if `need_to_sort_by_hilbert_curve` is true.
    pub fn find(&self, points : &mut Vec<Point>) -> LinkageResult {
        if self.need_to_sort_by_hilbert_curve {
            Point::hilbert_sort(points, self.bits_per_dimension as usize);
        }
        let mut distances = AdjacentPairDistance::all_pairs(points);

        if self.sort_distances_completely {
            self.find_by_sorting(points, &mut distances)
        }
        else {
            self.find_by_binning(points, &mut distances)
        }
    }

    /// Using a default sort, find the linkage distance by analyzing the distribution of distances between consecutive points 
    /// after they are arranged in Hilbert Curve order. Also set several other measures in the structure.
    /// 
    /// The default sort is O(N Log N) in performance and will yield a more accurate answer at the expense of a slower computation. 
    /// 
    ///   - points - Points to analyze. 
    ///   - distances - Distances between successive pairs of points.
    ///     This collection should have exactly one fewer entry than `points`.
    /// 
    /// NOTE: In my earlier C# code, this was called `FindMaximumSquareDistance`. 
    fn find_by_sorting(&self, points : &mut Vec<Point>, distances : &Vec<AdjacentPairDistance>) -> LinkageResult {
        // NOTE: This is a port of a C# method named `FindMaximumSquareDistance`, with mods.

        // Why clone? We will later need the unsorted distances to estimate the potential effect of a clustering. 
        let mut sorted_distances = distances.clone();
        
        // Use the default sort. 
        sorted_distances.sort();

        //println!("===== SORTED DISTANCES =======");
        //for dist in sorted_distances.iter() {
        //    println!("{}", dist.square_distance);
        //}
        //println!("==============================");

        let mut stats = DistanceGrowthStats::new();
        
        let start_index = 1 + self.noise_skip_by as usize + self.lowest_index_for_checking_growth_ratio as usize;
        let conservative_high_index = points.len() - self.minimum_cluster_count as usize;
        for i_distance in start_index..conservative_high_index
        {
            let distance = sorted_distances[i_distance].square_distance;
            let previous_index = i_distance - 1 - self.noise_skip_by as usize;
            let previous_distance = sorted_distances[previous_index].square_distance;
            stats.accumulate(i_distance, previous_distance, distance);
        }

        let index_to_use = stats.get_index_of_max_change(
            self.lowest_index_for_checking_growth_ratio as usize, 
            conservative_high_index
        );

        let maximum_square_distance = sorted_distances[index_to_use].square_distance;
        self.estimate_cluster_counts(distances, maximum_square_distance)
    }

    /// Find the linkage distance using logarithmic bins to perform a bin sort instead of the standard (likely iterative merge sort similar to Timsort). 
    /// 
    ///  - Use logarithmic bins to store the items initially
    ///  - Consolidate empty or sparse bins to guarantee a minimum bin size
    ///  - Find the bin where the average spread grows the most between successive bins (ruling out edge cases near the low end of the distances)
    ///  - Sort the values in that one bin to home in on the actual largest jump. 
    /// 
    ///   - `points` - Points whose distances
    /// 
    fn find_by_binning(&self, points : &mut Vec<Point>, hilbert_sorted_distances : &mut Vec<AdjacentPairDistance>) -> LinkageResult {

        // Part 1: Create bins for a logarithmic bucket sort, not the slower default sort 
        //         and gather the distances between points into bins. 
        let largest_possible_square_distance = 1_u64 << (2 * self.bits_per_dimension);
        let mut bins = DistanceBin::make_bins(20, largest_possible_square_distance, 20, 1.05);
        
        for pair in hilbert_sorted_distances.iter() {
            let bin_index = DistanceBin::find_bin(pair.square_distance, &mut bins);
            let bin = &mut bins[bin_index];
            bin.add(pair.square_distance);
        }

        // Part 2: Consolidate the buckets to combine consecutive empty or nearly empty buckets into one bucket. 
        let minimum_size = if self.noise_skip_by < 5 { 5 } else { self.noise_skip_by };
        bins = DistanceBin::consolidate(bins, minimum_size as usize);

        let mut index_of_maximum_increase = 0;
        let mut index_of_maximum_ratio = 0;
        let mut i_bin_of_maximum_increase = 0;
        let mut i_bin_of_maximum_ratio = 0;
        let mut max_increase = 0;
        let mut max_ratio = 0.0;
        let num_points = points.len();
        let mut cume_points = 0;

        // Part 3: Narrow the focus to a single bin likely to have the largest jump in distance.
        for (i_bin, bin) in bins.iter().enumerate() {
            let spread = bin.average_spread();
            let previous_spread = if i_bin == 0 { 0 } else { bins[i_bin - 1].average_spread() };
            let diff = spread - previous_spread;
            if diff > max_increase
            {
                max_increase = diff;
                index_of_maximum_increase = cume_points;
                i_bin_of_maximum_increase = i_bin;
            }
            if previous_spread > 1 && cume_points >= self.lowest_index_for_checking_growth_ratio as usize
            {
                let ratio = spread as f64 / previous_spread as f64;
                if ratio > max_ratio
                {
                    max_ratio = ratio;
                    index_of_maximum_ratio = cume_points;
                    i_bin_of_maximum_ratio = i_bin;

                    //TODO: The test "max_ratio > 5.0" could cause problems if we have clusters with highly varying densities.
                    //      Make it a parameter?
                    if cume_points > num_points / 2 && max_ratio > 5.0 { break; }
                }
            }
            cume_points += bin.len();
        }

        // Part 4: Choose which indicator of the largest jump in value is best for this case: absolute amount or relative ratio. 

        let i_bin_to_use =
            // If the two measures agree, we have an unambiguous choice.
            if index_of_maximum_increase == index_of_maximum_ratio {
                i_bin_of_maximum_increase
            }
            // If the highest ratio in length between one distance and the next is at an early index,
            // it is likely because we skipped from a really low value (like 1) to another really low value (like 10)
            // which only looks like a large jump because the values are so small.
            else if index_of_maximum_ratio < num_points / 2 {
                i_bin_of_maximum_increase
            }
            // Once we get near the end of the series of distances, the jumps between successive
            // distances can become large, but their relative change is small,
            // so rely on the ratio instead.
            else {
                i_bin_of_maximum_ratio
            };

        // Part 5: Analyze the selected bin to find the place where the distance grew the fastest. 
        // Do not use noise_skip_by to adjust index_to_use in this method, because the binning already smooths the curve. 
        // Sort the selected bin and find the place of the biggest jump with it. 
        //TODO: Sort and analyze selected bin. 
        let highest_value_from_previous_bin = if i_bin_to_use == 0 || bins[i_bin_to_use - 1].len() == 0 {
            bins[i_bin_to_use].bounds.start
        }
        else {
            bins[i_bin_to_use - 1].highest_value_added
        };
        let maximum_square_distance = bins[i_bin_to_use].find_square_distance_before_jump(highest_value_from_previous_bin); 

        let result = self.estimate_cluster_counts(hilbert_sorted_distances, maximum_square_distance);
        result
    }

    /// Estimate how many large clusters and outliers would be formed if we cluster using the
    /// given value of `linkage_square_distance` and a single pass through a set of points
    /// ordered by the Hilbert curve.
    /// 
    /// The values derived are upper bounds on:
    /// 
    ///   - large_cluster_count
    ///   - outlier_count
    ///   - outlier_cluster_count
    ///   - count_of_too_large_distances
    /// 
    /// After all clustering refinements are handled, all these numbers are likely to decline. 
    /// That is because the heuristic here will fail to merge together some smaller clusters
    /// that deserve to be merged into one. 
    /// 
    ///   - hilbert_sorted_distances - Distances between consecutive pairs of points that are 
    ///     sorted in Hilbert order (not by ascending distance)
    ///   - linkage_square_distance - Upper limit on distance between two points that permits them to be clustered together. 
    pub fn estimate_cluster_counts(&self, hilbert_sorted_distances : &Vec<AdjacentPairDistance>, linkage_square_distance : u64) -> LinkageResult {
        if linkage_square_distance == 0 { panic!("linkage_square_distance must be greater than zero."); }
        let mut linkage = LinkageResult::new();
        linkage.linkage_square_distance = linkage_square_distance;
        linkage.outlier_cluster_count = 0;
        linkage.outlier_count = 0;
        linkage.count_of_too_large_distances = 0;
        linkage.large_cluster_count = 0;

        let mut start_index_for_cluster = 0;
        let final_id = hilbert_sorted_distances.last().unwrap().second_id;
        for pair in hilbert_sorted_distances {
            if pair.square_distance > linkage_square_distance {
                // Close out current cluster and start a new one.
                let cluster_size = pair.second_index - start_index_for_cluster;
                if cluster_size <= self.outlier_cluster_size as usize {
                    linkage.outlier_cluster_count += 1;
                    linkage.outlier_count += cluster_size as u32;
                }
                else {
                    linkage.large_cluster_count += 1;
                }
                linkage.count_of_too_large_distances += 1;

                if pair.second_id == final_id {
                    // Finish off the last cluster, a single point. 
                    linkage.outlier_cluster_count += 1;
                    linkage.outlier_count += 1;
                }
                start_index_for_cluster = pair.second_index;
            }
            else if pair.second_id == final_id {
                let cluster_size = pair.second_index - start_index_for_cluster + 1; // +1 to Include last point.
                if cluster_size <= self.outlier_cluster_size as usize {
                    linkage.outlier_cluster_count += 1;
                    linkage.outlier_count += cluster_size as u32;
                }
                else {
                    linkage.large_cluster_count += 1;
                }
            }
        }
        linkage
    }
}

// ........................... DistanceGrowthStats .....................................................

#[derive(Clone, Debug)]
pub struct DistanceGrowthStats {
    index_of_maximum_increase : usize,
    index_of_maximum_ratio : usize,
    index_of_maximum_increase_and_ratio : usize,
    max_increase_alone : u64,
    max_ratio_alone : f64,
    max_increase_paired : u64,
    max_ratio_paired : f64
}

/// Internal struct for accumulating guesses as to where the curve formed by square distances between points grows the fastest.
impl DistanceGrowthStats {
    pub fn new() -> Self {
        DistanceGrowthStats {
            index_of_maximum_increase : 0,
            index_of_maximum_ratio : 0,
            index_of_maximum_increase_and_ratio : 0,
            max_increase_alone : 0,
            max_ratio_alone : 0.0,
            max_increase_paired : 0,
            max_ratio_paired : 0.0
        }
    }

    pub fn accumulate(&mut self, index : usize, previous_value : u64, new_value : u64) {
        let delta = new_value - previous_value;
        if previous_value == 0 { return; }
        let ratio = new_value as f64 / previous_value as f64;
        let mut both_high = true;
        if delta > self.max_increase_alone { 
            self.max_increase_alone = delta;
            self.index_of_maximum_increase = index;
        }
        else {
            both_high = false;
        }
        if ratio > self.max_ratio_alone { 
            self.max_ratio_alone = ratio;
            self.index_of_maximum_ratio = index;
        }
        else {
            both_high = false;
        }
        if both_high {
            self.index_of_maximum_increase_and_ratio = index;
            self.max_increase_paired = delta;
            self.max_ratio_paired = ratio;
        }
    }

    /// Decide when the distance value changed the most, but be conservative if several measures disagree.
    /// 
    ///   - `i_low_paired` - Do not choose `i_bin_of_maximum_increase_and_ratio` if it falls below this. 
    ///   - `i_high` - Do not go above this index.
    ///   - return - The index into the sorted pairs of points having the best guess for linkage distance.
    pub fn get_index_of_max_change(&self, i_low_paired : usize, i_high : usize) -> usize {
        let i_conservative = i_low_paired + (i_high - i_low_paired) * 3 / 4;
        if self.index_of_maximum_increase_and_ratio > i_high {
            i_high
        }
        else if self.index_of_maximum_increase_and_ratio > i_conservative {
            self.index_of_maximum_increase_and_ratio
        }
        else if self.index_of_maximum_ratio < i_conservative {
            max(min(i_high, self.index_of_maximum_increase), i_low_paired)
        }
        else if self.index_of_maximum_increase < i_conservative {
            max(min(i_high, self.index_of_maximum_ratio), i_low_paired)
        }
        else {
            min(min(i_high, self.index_of_maximum_increase), self.index_of_maximum_ratio)
        }
    }
}


// ........................... AdjacentPairDistance ..........................................

/// Measures the square distance between a pair of `Points` identified by their index into an ordered collection of `Points`.
#[derive(Copy, Clone, Debug)]
pub struct AdjacentPairDistance {
    /// Square of the distance between two points.
    pub square_distance : u64,
    /// Zero-based index of first point in Hilbert curve order. 
    pub first_index : usize,
    /// Zero-based index of second point in Hilbert curve order. 
    pub second_index : usize,
    /// Id of first point
    pub first_id : usize,
    /// Id of second point
    pub second_id : usize
}

impl Ord for AdjacentPairDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.square_distance, self.first_index, self.second_index).cmp(&(other.square_distance, other.first_index, other.second_index))
    }
}

impl PartialOrd for AdjacentPairDistance { fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) } }

impl PartialEq for AdjacentPairDistance {
    fn eq(&self, other: &Self) -> bool {
        (self.square_distance, self.first_index, self.second_index) == (other.square_distance, other.first_index, other.second_index)
    }
}

impl Eq for AdjacentPairDistance { }

impl AdjacentPairDistance {
    /// Construct a new AdjacentPairSequence.
    pub fn new(p1 : &Point, p2 : &Point, index1 : usize, index2 : usize) -> Self {
        AdjacentPairDistance { 
            square_distance : p1.square_distance(p2),
            first_index : index1,
            second_index : index2,
            first_id : p1.get_id(),
            second_id : p2.get_id()
        }
    }

    /// Generate a collection of the distances between consecutive points. 
    pub fn all_pairs(points : &Vec<Point>) -> Vec<AdjacentPairDistance> {
        if points.len() <= 1 { return Vec::new(); }
        let mut pairs = Vec::with_capacity(points.len() - 1);
        let mut previous = &points[0];
        for (index, current) in points.iter().skip(1).enumerate() {
            pairs.push(AdjacentPairDistance::new(previous, current, index, index + 1));
            previous = current;
        }
        pairs
    }
}

// ........................... DistanceBin ..........................................


/// Bin to hold unsorted values that fall within the bounds of the bin, as part of a bin sort. 
#[derive(Clone)]
pub struct DistanceBin {
    /// All values falling in this range can be added to this bin. 
    pub bounds : Range<u64>,

    /// Lowest of the values already added to this bin.
    pub lowest_value_added : u64,

    /// Highest of the values already added to this bin.
    pub highest_value_added : u64,

    /// Values that have been added to this bin. 
    pub values_added : Vec<u64>
}

impl DistanceBin {
    /// Create a new DistanceBin with a defined lower and upper bound of values that can be added to it.
    pub fn new(from : u64, to : u64) -> Self {
        DistanceBin { 
            bounds : from..to,
            lowest_value_added : to,
            highest_value_added : from,
            values_added : Vec::new()
        }
    }

    /// Make a series of bins of increasing size that goes from zero to top_of_highest_bin. 
    /// 
    ///   - `top_of_lowest_bin` - The first bin ranges from zero (inclusive) to this number (exclusive).
    ///   - `top_of_highest_bin` - The last bin will end with this value (exclusive). 
    ///   - `minimum_bin_width` - No bin will be narrower than this.
    ///   - `multiplier` - The top of each bin will equal the top of the previous bin times this multiplier,
    ///      unless that makes a bin whose width is less than `minimum_bin_width`. If this multiplier is 
    ///      less than 1.001, it will be set to 1.001. A good value is 1.05. 
    pub fn make_bins(top_of_lowest_bin : u64, top_of_highest_bin : u64, minimum_bin_width : u64, mut multiplier : f64) -> Vec<DistanceBin> {
        let mut bins = Vec::with_capacity(1000);
        if multiplier < 1.001 { multiplier = 1.001; }
        bins.push(DistanceBin::new(0, top_of_lowest_bin));
        let mut next_bottom = top_of_lowest_bin as f64;
        let mut next_top = (top_of_lowest_bin as f64 * multiplier).ceil();
        let tippy_top = top_of_highest_bin as f64;
        let min_width = minimum_bin_width as f64;
        while next_top < tippy_top {
            if next_top - next_bottom < min_width {
                next_top = next_bottom + min_width;
            }
            bins.push(DistanceBin::new(next_bottom as u64, next_top as u64));
            next_bottom = next_top;
            next_top = next_top * multiplier;
        }
        bins.push(DistanceBin::new(next_bottom as u64, next_top as u64));
        bins
    }

    /// Perform a binary search for the bin that contains the given value.
    /// Assumes that there are no gaps between bins.
    /// 
    ///   - value - Search for a bin that contains this value.
    ///   - bins - bins in sorted order (ascending). 
    pub fn find_bin(value : u64, bins : &mut Vec<DistanceBin>) -> usize {
        if bins.last_mut().unwrap().bounds.end <= value { return bins.len() - 1; }

        let bin_opt = bins.binary_search_by(
            |probe| {
                if value < probe.bounds.start { return Ordering::Greater; }
                if value > probe.bounds.end { return Ordering::Less; }
                return Ordering::Equal;
        });
        return bin_opt.unwrap();
    }

    /// Does the given square_distance fall within the bounds of the bin?
    pub fn is_in_bounds(&self, square_distance : u64) -> bool {
        self.bounds.contains(&square_distance)
    }

    /// Add an item to the bin if it falls within the bounds.
    /// 
    ///   - return true if able to add the value, false otherwise
    pub fn add(&mut self, square_distance : u64) -> bool {
        if !self.is_in_bounds(square_distance) { false }
        else {
            if square_distance > self.highest_value_added { self.highest_value_added = square_distance; }
            if square_distance < self.lowest_value_added { self.lowest_value_added = square_distance; }
            self.values_added.push(square_distance);
            true
        }
    }

    /// Fluently merge two bins by widening the given DistanceBin to accommodate the full range between it and higher_bin. 
    /// 
    ///   - `higher_bin` - Must enclose a range that starts and ends higher than the given bin's range.
    ///   - return - The same bin, with changes.
    pub fn merge(mut self, higher_bin : &Self) -> Self{
        if higher_bin.bounds.start < self.bounds.end {
            panic!("The second bin in a merge must have bounds that come after those of the first bin.");
        }
        self.bounds = self.bounds.start .. higher_bin.bounds.end;
        if higher_bin.len() > 0 {
            self.values_added.extend(higher_bin.values_added.iter());
            self.highest_value_added = higher_bin.highest_value_added;
        }
        self
    }

    /// Sort the items contained in the bin. 
    pub fn sort(&mut self) {
        self.values_added.sort();
    }

    /// Move all bins from one vector to another, but consolidate consecutive runs of bins into a single bin if they hold too few items.
    /// 
    /// The search for the linkage distance requires that we have several items in a bin in order to decide if the contained values have jumped a lot. 
    /// 
    ///   - `original_bins` - Bins to be consolidated. 
    ///   - `minimum_size` - If a bin does not contain at least this many items, merge it with the following bin. 
    ///                      Continue this repeatedly until either the bin grows large enough or we reach the last bin
    ///                      in the Vec. 
    pub fn consolidate(original_bins : Vec<Self>, minimum_size : usize) -> Vec<Self> {
        let mut consolidated_bins = Vec::with_capacity(original_bins.len());
        let mut hold_bin_opt : Option<Self> = None;
        for bin in original_bins {
            match hold_bin_opt {
                Some (ref hold_bin) => {
                    let merged = bin.merge(hold_bin);
                    if merged.len() >= minimum_size {
                        consolidated_bins.push(merged);
                        hold_bin_opt = None;
                    }
                    else { hold_bin_opt = Some(merged); }
                },
                None => {
                    if bin.len() >= minimum_size { consolidated_bins.push(bin); }
                    else { hold_bin_opt = Some(bin); }
                }
            }
        }
        if let Some(hold_bin) = hold_bin_opt {
            consolidated_bins.push(hold_bin);
        }
        consolidated_bins
    }

    /// Number of values that have been added to the bin.
    pub fn len(&self) -> usize {
        self.values_added.len()
    }

    /// Average difference between consecutive points within the bin. 
    /// 
    ///   - If DistanceBin has zero or one points, the spread is width of the bounds. 
    ///   - Otherwise, it is `(highest_value_added - lowest_value_added) / (len() - 1))`.
    pub fn average_spread(&self) -> u64 {
        if self.len() <= 1 { self.bounds.end - self.bounds.start }
        else { (self.highest_value_added - self.lowest_value_added) / (self.len() as u64 - 1) }
    }

    /// Sort the values_added in the bin by ascending value and find the place where the value jumps the most. 
    /// Return the value prior to the jump.
    pub fn find_square_distance_before_jump(&mut self, highest_value_from_previous_bin : u64) -> u64 {
        if self.values_added.len() == 0 { return self.bounds.start; }
        if self.values_added.len() <= 2 { return self.lowest_value_added; }
        self.sort();
        let mut value_before_biggest_jump = highest_value_from_previous_bin;
        let mut high_delta = self.lowest_value_added - highest_value_from_previous_bin;
        let mut previous_value = highest_value_from_previous_bin;
        for value in self.values_added.iter() {
            let delta = value - previous_value;
            if delta > high_delta {
                value_before_biggest_jump = previous_value;
                high_delta = delta;
            }
            previous_value = *value;
        }
        value_before_biggest_jump
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use std::cmp::Ordering;
    use spectral::prelude::*;
    use super::AdjacentPairDistance;

    #[test]
    fn adjacent_pair_distance_cmp(){
        let pair1 = AdjacentPairDistance { 
            square_distance : 100,
            first_index : 1,
            second_index : 2,
            first_id : 1,
            second_id : 2
        };
        let pair2 = AdjacentPairDistance { 
            square_distance : 50,
            first_index : 2,
            second_index : 3,
            first_id : 2,
            second_id : 3
        };
        let comparison = pair1.cmp(&pair2);
        asserting("Should compare greater than").that(&(comparison == Ordering::Greater)).is_equal_to(true);
    }

}
