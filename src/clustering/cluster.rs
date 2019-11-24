use std::collections::HashSet;
use std::cmp::{Ord,Ordering};
use std::hash::Hash;

use std::fmt::{Formatter,Debug,Result};

/// Combination Trait for Cluster keys and members, which must implement **C**opy, **H**ash, **O**rd, **P**artialEq and **D**ebug (C.H.O.P.E.D).
pub trait Chopped : Copy + Hash + Ord + PartialEq + Eq + Debug {}

/// Blanket duck-type implementation of Chopped
impl<T: Copy + Hash + Ord + PartialEq + Eq + Debug> Chopped for T {}

/// Groups zero or more members into a category as part of a Clustering.
/// 
/// While the members could be Copyable structs, typically they are integer ids the caller can use to obtain the real objects.
/// 
///   - C : Type of the Cluster Category. 
///   - M : Type of the Cluster Members. 
#[derive(Clone)]
pub struct Cluster<C : Chopped, M : Chopped>
{
    /// Category for the cluster.
    category : C,

    /// Members of the Cluster.
    members : HashSet<M>
}

impl<C : Chopped, M : Chopped> Cluster<C, M> {
    /// Create an empty cluster.
    pub fn empty(category : C) -> Self {
        Cluster { category, members : HashSet::new() }
    }

    /// Create a cluster containing a single member.
    pub fn with_member(category : C, member : M) -> Self {
        let mut cluster = Cluster { category, members : HashSet::new() };
        cluster.members.insert(member);
        cluster
    }

    /// Count of members in the cluster
    pub fn len(&self) -> usize { self.members.len() }

    /// Is the `Cluster` empty?
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Get the Cluster category.
    pub fn get_category(&self) -> C { self.category } 

    /// Get the members of the cluster.
    pub fn get_members(&self) -> &HashSet<M> { &self.members }

    /// Determines if the given item is a member of the cluster.
    pub fn is_member(&self, item : &M) -> bool { self.members.contains(item) }

    /// Adds a new member to the cluster.
    ///   - returns true if a new item was added, 
    ///   - returns false if the item was already present in the Cluster.
    pub fn add_member(&mut self, item : M) -> bool { self.members.insert(item) }

    /// Removes a member from the cluster.
    ///   - returns true item was removed, 
    ///   - returns false if the item is not present in the Cluster.
    pub fn remove_member(&mut self, item : &M) -> bool { self.members.remove(item) }

    /// Merge two clusters, removing all members from the second and inserting them into the first. 
    pub fn merge(&mut self, other : &mut Self) {
        self.members.extend(other.members.iter());
        other.members.clear();
    }
}

impl<C : Chopped, M : Chopped> Debug for Cluster<C, M> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut member_list : Vec<M> = self.members.iter().map(|m| *m).collect();
        member_list.sort();
        let member_string_list : Vec<String> = member_list.iter().map(|m| format!("{:?}", m)).collect();
        write!(f, "Cluster for category '{:?}' has {} members: {}", self.category, self.members.len(), member_string_list.join(","))
    }
}

impl<C : Chopped, M : Chopped> PartialEq for Cluster<C, M> {
    fn eq(&self, other: &Self) -> bool {
        self.category == other.category && self.members.len() == other.members.len()
    }
}

impl<C : Chopped, M : Chopped> Eq for Cluster<C, M> {}

impl<C : Chopped, M : Chopped> Ord for Cluster<C, M> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Multi-column sort by category, then member count. 
        (self.category,self.members.len()).cmp(&(other.category, other.members.len()))
    }
}

impl<C : Chopped, M : Chopped> PartialOrd for Cluster<C, M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
