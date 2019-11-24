use std::collections::{HashMap, hash_map::Entry, HashSet};
use super::cluster::{Cluster, Chopped};
use super::Clustering;

/// The B-Cubed extrinsic measure of the similarity of two Clusterings. 
/// 
/// A similarity of one means perfect concordance between clusters and gold-standard truth set categories.
/// The closer the similarity gets to zero, the worse the concordance. 
/// 
/// The B-Cubed measure was proposed in this paper:
/// 
/// [1] **A. Bagga and B. Baldwin**. _Entity-based cross-document coreferencing using the vector space model_.
/// In Proceedings of the 36th Annual Meeting of the Association for Computational Linguistics - 
/// Volume 1, ACL ’98, pages 79–85, 1998.
/// 
/// There are many measures of clustering accuracy, some better than others. 
/// The following paper compared many of them and found B-Cubed the best according to four formal constraints:
/// 
///   1. Cluster Homogeneity
///   2. Cluster Completeness
///   3. Rag Bag
///   4. Cluster Size vs quantity
/// 
/// [2] _A comparison of Extrinsic Clustering Evaluation Metrics based on Formal Constraints_ 
///     by **Enrique Amigo, Julio Gonzalo, Javier Artiles, Felisa Verdejo**
///     of the Departamento de Lenguajes y Sistemas Informaticos UNED, Madrid, Spain, May 11, 2009
/// 
/// A subsequent paper identified a use case where B-Cubed fared poorly: unbalanced datasets where one cluster dominates: 
/// 
/// [3] _Adapted B-CUBED Metrics to Unbalanced Datasets_ by Jose G. Moreno and Gaël Dias, 
///     both of Normandie University in France. 
/// 
/// This third paper proposed a refined version of B-Cubed, but the added complexity adds significantly to processing time, 
/// so those refinements are not employed here. The definition of the algorithm used here is taken from section 2.1 of this 
/// last paper. (The refined version is in section 2.2.)
/// 
/// ```
///  //   𝔽 = F-measure (final similarity measure)
///  //   ℙ = Precision (a measure of homogeneity)
///  //   ℝ = Recall (a measure of completeness)
///  //   α = Weighting factor (defaults to 0.5)
///  //   ℕ = Number of points
///  //   k = Number of categories (varies between the π and π* Clusterings)
///  //   i = category index
///  //  πᵢ = cluster solution for the ith category
///  //  π*ᵢ= gold standard for the ith category
///  //  g₀ = tests whether two items share the same category in the clustering
///  //  g*₀= tests whether two items share the same category in the gold standard
///  //
///  //      𝟙       α     𝟙 - α
///  //    ━━━━━ ═ ━━━━━ + ━━━━━
///  //     𝔽       ℙ       ℝ
///  //      b³      b³      b³
///  //
///  //                     k
///  //     ℙ         𝟙    ⎲     𝟙     ⎲   ⎲    
///  //      b³  ═   ━━━   ⎳   ━━━━━   ⎳   ⎳   g*₀(xⱼ,xₗ)
///  //               ℕ    i=1   |πᵢ|   xⱼ∈πᵢ xₗ∈πᵢ
///  //
///  //                     k
///  //     ℝ         𝟙    ⎲     𝟙     ⎲    ⎲    
///  //      b³  ═   ━━━   ⎳   ━━━━━   ⎳    ⎳   g₀(xⱼ,xₗ)
///  //               ℕ    i=1   |π*ᵢ|  xⱼ∈π*ᵢ xₗ∈π*ᵢ
///  //
///  //              (  𝟙 ⟺ ∃l:xᵢ∈πₗ ∧ xⱼ∈πₗ
///  // g₀(xᵢ,xⱼ)  ═ <
///  //              (  𝟘, otherwise
///  //
///  //
///  //              (  𝟙 ⟺ ∃l:xᵢ∈π*ₗ ∧ xⱼ∈π*ₗ
///  // g*₀(xᵢ,xⱼ) ═ <
///  //              (  𝟘, otherwise
/// 
/// 
/// ```
pub struct BCubed {
    /// `Precision` is a measure of homogeneity. 
    /// Are only related objects grouped together (high precision), 
    /// or are they mixed with unrelated ones (low precision)?)
    precision : f64,

    /// `Recall` is a measure of completeness. 
    /// Are related objects gathered into a single group (high recall) 
    /// or split in separate clusters (low recall)?
    recall : f64,

    /// `alpha` is used to combine `precision` and `recall` into a single similarity measure.
    /// It ranges from zero to one. 
    ///    - If `alpha` is 0.5, `precision` and `recall` are weighted equally. 
    ///    - If `alpha` is zero, only `recall` is used. 
    ///    - If `alpha` is one, only `precision` is used. 
    alpha : f64
}

impl BCubed {
    /// Create a BCubed value, knowing all its components. 
    pub fn new(precision : f64, recall : f64, alpha : f64) -> Self {
        BCubed { precision, recall, alpha }
    }

    /// Get the precision, a measure of homogeneity from zero to one. 
    pub fn get_precision(&self) -> f64 { self.precision }

    /// Get the recall, a measure of completeness from zero to one. 
    pub fn get_recall(&self) -> f64 { self.recall }

    /// Get alpha, the weighting factor that ranges between zero and one 
    /// and can shift between favoring Precision or Recall in the 
    /// similarity calculation.
    pub fn get_alpha(&self) -> f64 { self.alpha }

    /// The F-measure (a harmonic average) applied to precision and recall,
    /// a unified measure of the quality of the clustering.
    pub fn similarity(&self) -> f64 {
        self.precision * self.recall / (self.alpha * self.recall + (1.0 - self.alpha) * self.precision)
    }

    /// Compare two Clusterings and compute the BCubed value.
    /// 
    ///   - solution - The `Clustering` whose quality is to be assessed.
    ///   - gold_standard - The perfect `Clustering` whose categories are all properly assigned. 
    ///   - alpha - A value between zero and one. used to weight `precision` and `recall`. 
    ///        - If `alpha` is 0.5, `precision` and `recall` are weighted equally. 
    ///        - If `alpha` is zero, only `recall` is used. 
    ///        - If `alpha` is one, only `precision` is used. 
    pub fn compare<C : Chopped, M : Chopped, G : Iterator<Item = C>>(solution : &Clustering<C,M,G>, gold_standard : &Clustering<C,M,G>, alpha : f64) -> Self {
        BCubed::new(
            BCubed::compute_precision(solution, gold_standard), 
            BCubed::compute_recall(solution, gold_standard), 
            alpha)
    }

    /// Compute the BCubed Precision.
    fn compute_precision<C : Chopped, M : Chopped, G : Iterator<Item = C>>(solution : &Clustering<C,M,G>, gold_standard : &Clustering<C,M,G>) -> f64 {
        let n = solution.member_count() as f64;
        let mut weighted_sum = 0_f64;
        for cluster in solution.get_clusters().values() {
            let pi_sub_i_magnitude = cluster.len() as f64;
            let sum_of_squares = Self::tally_squares(
                cluster.get_members()
                       .iter()
                       .map(|m| gold_standard.get_category(*m)
                           .expect(&format!("Item {:?} from one Clustering not present in the other", *m)))
            ) as f64;
            weighted_sum += sum_of_squares / pi_sub_i_magnitude;
        }
        weighted_sum / n
    }
    /// Compute the BCubed Recall.
    fn compute_recall<C : Chopped, M : Chopped, G : Iterator<Item = C>>(solution : &Clustering<C,M,G>, gold_standard : &Clustering<C,M,G>) -> f64 {
        // The computation for Recall is the symmetric with that of Precision; we just swap the 
        // order of the Clusterings.
        Self::compute_precision(gold_standard, solution)
    }

    /// Count how many times each category appears in the Iterator, and sum the squares of the number of counts. 
    /// 
    /// This computation is equivalent to a quadratic, doubly-nested loop over all items that compares
    /// the category values of each pair of items and adds one if they match. 
    /// This algorithm requires only a single pass through the items and their categories, so **reduces
    /// the complexity from quadratic to linear**. 
    /// In the formula for B-Cubed, this performs the parts notated as:
    ///   - Σ Σ g*₀(xⱼ,xₗ)
    ///   - Σ Σ g₀(xⱼ,xₗ)
    /// 
    /// It only does so for a single cluster. The caller must loop over all clusters.
    fn tally_squares<C : Chopped, I : Iterator<Item = C>>(categories : I) -> u64 {
        let mut sum_of_squares = 0_u64;
        let mut tallies : HashMap<C, u64> = HashMap::new();
        for category in categories {
            match tallies.entry(category) {
                Entry::Occupied(mut entry) => {
                    let current_tally = *entry.get();
                    sum_of_squares += 2 * current_tally + 1;
                    *entry.get_mut() = current_tally + 1;
                },
                Entry::Vacant(entry) => {
                    sum_of_squares += 1;
                    entry.insert(1);
                }
            }
        }
        sum_of_squares
    }
}
