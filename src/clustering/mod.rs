use std::collections::{HashMap};
use std::fmt::{Formatter,Debug};
use std::ops::Range;
use std::usize;
pub mod cluster;
pub mod bcubed;
pub mod single_linkage;

use self::cluster::{Cluster, Chopped};

/// Partitions items into one or more non-overlapping Clusters. 
/// Each item may belong to a single Cluster.
/// 
/// Clusters may be combined using the `merge` method. 
pub struct Clustering<C : Chopped, M : Chopped, G : Iterator<Item = C>> {
    /// Associates each member with the category of the Cluster of which it is currently a member.
    member_to_cluster : HashMap<M,C>,

    /// Associates each category with the Cluster that holds all items that belong to that category.
    clusters : HashMap<C, Cluster<C,M>>,

    /// Callback to generate new Cluster categories.
    /// 
    /// Typically this is an auto-incrementing integer function.
    category_generator : G
}

impl<C : Chopped, M : Chopped, G : Iterator<Item = C>> Clustering<C, M, G> {
    /// Create an empty Clustering. 
    pub fn empty(category_generator : G) -> Self {
        Clustering {
            member_to_cluster : HashMap::new(),
            clusters : HashMap::new(),
            category_generator
        }
    }

    /// Create an Clustering with many items, each a member of its own Cluster.
    pub fn uncategorized<I : Iterator<Item = M>>(items : &mut I, category_generator : G) -> Self {
        let mut clustering = Clustering::empty(category_generator);
        for member in items {
            clustering.add_to_new_cluster(member).expect(&format!("Unable to add item {:?} to new cluster", member));
        }
        clustering
    }

    /// Create a new `Cluster` in the `Clustering` and add the given item to it.
    /// 
    ///   - returns - An `Ok` containing the category of the new `Cluster` 
    ///     if the item was not previously present in the `Clustering` and was able to be added to a new `Cluster`,
    ///   - returns - An `Err` containing the category of the existing `Cluster` if the item is already clustered
    ///   - panics - If unable to generate any new categories. 
    pub fn add_to_new_cluster(&mut self, item : M) -> Result<C,C> {
        match self.get_category(item) {
            Some(current_category) => Err(current_category),
            None => {
                match self.category_generator.next() {
                    Some(category) => {
                        let cluster = Cluster::with_member(category, item);
                        self.clusters.insert(category, cluster);
                        self.member_to_cluster.insert(item, category);
                        Ok(category)
                    },
                    None => panic!("category_generator ran out of new categories")
                }
            }
        }
    }

    /// Add the given not-yet-clustered item to the existing `Cluster` for the given category.
    /// 
    ///   - returns - An `Ok` containing the given category  
    ///     if the item was not previously present in the `Clustering` and was able to be added to the `Cluster`,
    ///   - returns - An `Err` containing the category of the existing `Cluster` if the item is already clustered
    ///   - panics - If there is no such category in the Clustering.
    pub fn add_to_cluster(&mut self, item : M, category : C) -> Result<C,C> {
        match self.get_category(item) {
            Some(current_category) => Err(current_category),
            None => {
                match self.clusters.get_mut(&category) {
                    Some(cluster) => {
                        cluster.add_member(item);
                        self.member_to_cluster.insert(item, category);
                        Ok(category)
                    },
                    None => panic!("get_category inconsistent with clusters")
                }
            }
        }
    }

    /// Merge the `Cluster` holding item1 with the `Cluster` holding item2. 
    /// 
    /// The merge is _transitive_; all members affiliated with item1 will now be in the same cluster as all members affiliated with item2.
    /// 
    ///  1. If the items are already clustered together, no change occurs. 
    ///  2. If both items are already present in the `Clustering`, merge together all items from both Clusters into the Cluster holding item1. 
    ///  3. If neither item is currently in a Cluster, create a new Cluster and add each. 
    ///  4. If one item is in a Cluster and the other is not, add the unclustered item to the Cluster of the other.
    ///  5. Returns:
    ///     - false if the items are already clustered together
    ///     - true if the items are not already clustered together
    pub fn merge(&mut self, item1 : M, item2 : M) -> bool {
        let categories = (self.get_category(item1), self.get_category(item2));
        match categories {
            (Some(category1), Some(category2)) => {
                if category1 == category2 {return false; }
                // True merge. Remove all items from cluster2 and insert them into cluster1 and update indices.
                let mut cluster2_members : Vec<M> = Vec::new();
                {
                  let cluster2 = self.get_cluster(category2).unwrap();
                  cluster2_members.extend(cluster2.get_members().iter().map(|m| m.clone()));
                }
                for member in cluster2_members.iter() {
                    self.member_to_cluster.insert(*member, category1);
                }
                // I would prefer to perform this loop step in the same loop as above, but the borrow checker prevents.
                let cluster1 = self.get_cluster_mut(category1).unwrap();
                for member in cluster2_members.iter() {
                    cluster1.add_member(*member);
                }
                
                self.clusters.remove(&category2);
            },
            (Some(category1), None) => { self.add_to_cluster(item2, category1).expect(&format!("Unable to add item to category {:?}", category1)); },
            (None, Some(category2)) => { self.add_to_cluster(item1, category2).expect(&format!("Unable to add item to category {:?}", category2)); },
            (None, None) => {
                let new_category = self.add_to_new_cluster(item1).unwrap();
                let _ = self.add_to_cluster(item2, new_category);
            }
        }
        true
    }

    /// Remove an item from its `Cluster` and from its place in the index for the `Clustering`. 
    /// 
    /// If the item is the last item in its `Cluster`, that `Cluster` is removed from the `Clustering`. 
    /// 
    ///   Returns:
    ///     - `true` if able to find the item and remove it
    ///     - `false` if unable to find the item in `Clustering`
    pub fn remove_item(&mut self, item : M) -> bool {
        match self.get_category(item) {
            Some(category) => {
                {
                    let cluster = self.get_cluster_mut(category).expect(&format!("No Cluster for category {:?}", category));
                    cluster.remove_member(&item);
                    if cluster.is_empty() { self.clusters.remove(&category); }
                }
                self.member_to_cluster.remove(&item).expect(&format!("Member {:?} not in Clustering index", item));
                true
            },
            None => false
        }
    }

    /// Move an item to a different (but existing) category. 
    /// 
    /// Unlike merge, only the given item is moved; any items with which it had been clustered remain behind in their `Cluster`.
    /// 
    ///   1. If `new_category` does not exist in the `Clustering`, return false and do not remove the `item`. 
    ///   2. If the item was not previously present in the `Clustering`, 
    ///      add it to the `Cluster` for `new_category` and return true.
    ///   3. If the item was already in `new_category`, do nothing and return false. 
    ///   4. Otherwise, remove the item from its current category and add it to `new_category`. 
    /// 
    ///   - Returns true if a change was made, false otherwise. 
    pub fn move_item(&mut self, item : M, new_category : C) -> bool {
        if !self.contains_category(new_category) { return false; }
        match self.get_category(item) {
            Some(current_category) if current_category == new_category  => {
                false
            },
            Some(current_category) if current_category != new_category  => {
                self.remove_item(item);
                self.add_to_cluster(item, new_category).expect(&format!("Unable to add item to new category {:?}", new_category));
                true
            },
            None => {
                self.add_to_cluster(item, new_category).expect(&format!("Unable to add item to new category {:?}", new_category));
                true
            },
            _ => panic!("Impossible case in move_item")
        }
    }

    /// Check if the Clustering contains the given item in any of its Clusters. 
    pub fn contains_item(&self, item : M) -> bool { self.member_to_cluster.contains_key(&item) }

    /// Check if the `Clustering` contains a `Cluster` for the given category. 
    pub fn contains_category(&self, category : C) -> bool { self.clusters.contains_key(&category) }

    /// Get the category for the `Cluster` with which the given item is grouped.
    /// 
    ///   - returns - Some(category), if the item is clustered. 
    ///   - returns - None, if the item is not present in the Clustering.
    pub fn get_category(&self, item : M) -> Option<C> { self.member_to_cluster.get(&item).map_or_else(|| None, |c| Some(*c)) }

    /// Get the cluster for the given category.
    /// 
    ///   - returns - `Some(Cluster)`, if the category is present in the `Clustering`. 
    ///   - returns - `None`, if the category is not present in the `Clustering`.
    pub fn get_cluster(&self, category : C) -> Option<&Cluster<C,M>> { self.clusters.get(&category) }

    /// Get all the `Clusters` in the `Clustering`.
    pub fn get_clusters<'a>(&self) -> &HashMap<C,Cluster<C,M>> {
        &self.clusters
    }

    /// Get the cluster for the given category.
    /// 
    ///   - returns - `Some(Cluster)`, if the category is present in the `Clustering`. 
    ///   - returns - `None`, if the category is not present in the `Clustering`.
    pub fn get_cluster_mut(&mut self, category : C) -> Option<&mut Cluster<C,M>> { self.clusters.get_mut(&category) }

    /// Check if both items are grouped into the same Cluster. 
    /// 
    ///   - returns - `true`, if both items are present in the Clustering and grouped into the same `Cluster`.
    ///   - returns - `false`, if either item is not present in the `Clustering` or if they are in separate `Clusters`.
    pub fn are_together(&self, item1 : M, item2 : M) -> bool {
        match (self.get_category(item1), self.get_category(item2)) {
            (Some(category1), Some(category2)) => category1 == category2,
            _ => false
        }
    }

    /// Number of Clusters into which items are partitioned
    pub fn cluster_count(&self) -> usize { self.clusters.len() }

    /// Number of members in all the Clusters combined.
    pub fn member_count(&self) -> usize { self.member_to_cluster.len() }
}

/// Create a Clustering where the Cluster categories and Members are usize. 
/// 
/// When the members are usize, it usually means that the real objects being categorized are stored elsewhere, 
/// such as in a Vec or HashMap, with the member being the index into that collection. 
pub fn integer_clustering() -> Clustering<usize,usize,Range<usize>> {
    Clustering::empty(0..usize::MAX)
}

/// Deserialize a Clustering from a string that has positive integers grouped into clusters 
/// using commas to separate numbers within a cluster
/// and semicolons to separate clusters. 
/// 
/// All resulting clusters will be numbered sequentially; the string does not hold cluster ids, just member ids. 
/// 
///   - `clustering_string` - Clusters integers together. 
/// 
/// Example with four clusters: 
/// 
/// ```
///    use clusterphobia::clustering::{Clustering,from_delimited_string};
///    let clustering = from_delimited_string("1,2,3;4,5,6;7,8,9;10");
///    assert_eq!(clustering.cluster_count(), 4);
///    assert_eq!(clustering.member_count(), 10);
/// ```
/// 
/// Panics on bad input.
/// 
/// NOTE: This method is most useful for assembling test data concisely. 
pub fn from_delimited_string(clustering_string : &str) -> Clustering<usize,usize,Range<usize>> {
    let mut clustering = integer_clustering();
    for cluster_string in clustering_string.split(';') {
        let mut cluster_id_opt = None;
        for member_string in cluster_string.split(',') {
            let member = member_string.parse::<usize>().unwrap();
            match cluster_id_opt {
                Some(cluster_id) => { clustering.add_to_cluster(member, cluster_id).expect(&format!("Unable to add {} to cluster {}", member, cluster_id)); },
                None => { cluster_id_opt = Some(clustering.add_to_new_cluster(member).expect(&format!("Unable to add {} to a new cluster", member))); }
            }
        }
    }
    clustering
}

impl<C : Chopped, M : Chopped, G : Iterator<Item = C>> Debug for Clustering<C, M, G> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut clusters_formatted = String::new();
        for cluster in self.clusters.values() {
            clusters_formatted.push_str(&format!("  {:?}\n", cluster));
        }
        write!(f, "Clustering {} members into {} clusters\n{}", self.member_count(), self.cluster_count(), clusters_formatted)
    }
}


#[cfg(test)]
/// Tests of the Clustering methods.
mod tests {
    #[allow(unused_imports)]
    use spectral::prelude::*;
    use crate::clustering;

    #[test]
    fn from_delimited_string() {
        let clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        assert_eq!(clustering.cluster_count(), 4);
        assert_eq!(clustering.member_count(), 10);
        asserting("Membership").that(&clustering.get_category(4).unwrap()).is_equal_to(1);
    }

    #[test]
    fn are_together() {
        let clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        asserting("Should be together").that(&clustering.are_together(7,9)).is_equal_to(true);
        asserting("Should be apart").that(&clustering.are_together(2,4)).is_equal_to(false);
    }

    #[test]
    fn get_category() {
        let clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        asserting("Membership").that(&clustering.get_category(8).unwrap()).is_equal_to(2);
    }

    #[test]
    fn contains_item() {
        let clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        asserting("Should contain item").that(&clustering.contains_item(8)).is_equal_to(true);
        asserting("Should not contain item").that(&clustering.contains_item(11)).is_equal_to(false);
    }

    #[test]
    fn add_to_new_cluster() {
        let mut clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        clustering.add_to_new_cluster(100).expect("Unable to add to new cluster");
        asserting("Should contain item").that(&clustering.contains_item(100)).is_equal_to(true);
        asserting("Membership").that(&clustering.get_category(100).unwrap()).is_equal_to(4);
    }

    #[test]
    fn add_to_cluster() {
        let mut clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        clustering.add_to_cluster(100, 1).expect("Unable to add to existing cluster");
        asserting("Should contain item").that(&clustering.contains_item(100)).is_equal_to(true);
        asserting("Membership").that(&clustering.get_category(100).unwrap()).is_equal_to(1);
    }    

    #[test]
    fn merge() {
        let mut clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        asserting("Should merge").that(&clustering.merge(1, 10)).is_equal_to(true);
        asserting("Should be together").that(&clustering.are_together(2,10)).is_equal_to(true);
        assert_eq!(clustering.cluster_count(), 3);
    }

    #[test]
    fn move_item() {
        let mut clustering = clustering::from_delimited_string("1,2,3;4,5,6;7,8,9;10");
        clustering.move_item(6, 0);
        asserting("Membership changed").that(&clustering.get_category(6).unwrap()).is_equal_to(0);
        asserting("Membership unchanged").that(&clustering.get_category(5).unwrap()).is_equal_to(1);
    }    
}
