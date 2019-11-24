# clusterphobia - A Rust Crate for People Who Fear Clustering

This crate is based on ideas and algorithms originally developed in C# in this repository:

https://github.com/paulchernoch/HilbertTransformation

For an understanding of the theory, see **SLASH.pdf** in the `clusterphobia` github repo:

https://github.com/paulchernoch/clusterphobia

## Project Goals

**Clusterphobia** is intended to provide clustering tools for _scalable_, _high-dimensional_, _unassisted_, _flat_, _single classification_ of data.

  - **Scalable**: A bedrock feature of the library is the use of the Hilbert Curve (from the **hilbert** crate) to speed up **k-nearest neighbor searches**. This algorithm is _linear_ in the number of dimensions, whereas many others grow _exponentially_ with the number of dimensions. Parts of the algorithm have running times proportional to N Log N, where N is the number of points being clustered.
  - **High-dimensional** - The algorithm will work with up to ten thousand dimensions.
  - **Unassisted** - Clustering is performed on unlabeled data.
  - **Flat** - The algorithms group items into flat, mutually exclusive sets of categories, not hierarchies. 
  - **Single** - Each item is assigned a single category. Items cannot be part of multiple categories.

Two main approaches will be used to cluster data:

  - bottom-up, single-link agglomerative clustering
  - bottom-up, density-based clustering

## Functional Areas

Once complete, the crate will have these main areas of functionality:

  - **Data preparation** : Adjust raw data until the coordinate values are commensurate, non-negative integers. The adjustments include scaling, shifting and weighting each dimension. For documents modeled as bags-of-words, a novel, randomized transformation will be used. (Some of the transformations are supplied by the **Hilbert** crate.)
  - **Point representation** : Convert the prepared data into `Points` suitable for the clustering algorithms. (The `Point` structure is defined in the **Hilbert** crate and employs an **optimized Euclidean distance** formula, critical for nearest neighbor searches.)
  - **Linkage analysis** - The `Point` data is analyzed (using the **Hilbert curve**) and the linkage distance and density threshold are derived. These values are crucial for the clustering algorithms. (Two points separated by less than the **linkage distance** are assumed to belong to the same cluster. If a set number of points fall inside a region whose radius does not exceed the **density threshold**, the `Point` density is considered high enough to force a clustering.)
  - **Cluster representation** - A `Clustering` holds one `Cluster` for each category in the partitioning scheme. It supports operations like adding points to clusters, recategorizing them, and merging clusters. The `Clustering` is the principal output of the clustering process.
  - **Cluster Similarity** - To test and iteratively refine choices of dimensions, scaling factors, thresholds and tunable parameters, you need to be able to compare a perfect, assisted solution to the unassisted clustering. This crate provides an 
  optimized implementation of the **B-Cubed** measure with a linear (not a quadratic) runtime.
  - **Clustering algorithms** - These algorithms will arrange a collection of `Points` into a `Clustering`.
  - **Problem data** - The same test data sets show up in numerous papers, because they contain features that defeat many clustering algorithms, such as spirals, clusters with noise or partial overlaps. The algorithms in this crate will be tuned to be able to address as many as possible of these problems.

## Currently Working

The following features are ready for use:

  - Some data preparation transforms (from the **hilbert** crate), including `IntegerDataRange` and `FloatDataRange`.
  - Hilbert Curve Transformation, permutations and sorting (from the **hilbert** crate)
  - `Point` struct with optimized distance formula (from the **hilbert** crate)
  - `Clustering` struct which can be used to build and modify classification schemes.
  - `BCubed` struct which can represent a _similarity_ measure and compute the similarity between two clusters (essential for unit tests and tuning).

## Cluster Similarity

The literature is teaming with algorithms for cluster similarity. None handle all the edge cases perfectly. A measure that handles many commons use cases well and is not too expensive to run is called **B-Cubed**, named after initials taken from the names of its creators. Since initially proposed, others have published modifications to it.

The B-Cubed measure was proposed in this 1998 paper:
 
[1] **A. Bagga and B. Baldwin**. _Entity-based cross-document coreferencing using the vector space model_. In Proceedings of the 36th Annual Meeting of the Association for Computational Linguistics - Volume 1, ACL ’98, pages 79–85, 1998.
 
The following paper compared many clustering algorithms and found **B-Cubed** the best according to four formal constraints: 

   1. Cluster Homogeneity
   2. Cluster Completeness
   3. Rag Bag
   4. Cluster Size vs quantity
 
[2] _A comparison of Extrinsic Clustering Evaluation Metrics based on Formal Constraints_  by **Enrique Amigo, Julio Gonzalo, Javier Artiles, Felisa Verdejo**
of the Departamento de Lenguajes y Sistemas Informaticos UNED, Madrid, Spain, May 11, 2009

The above is available here: http://nlp.uned.es/docs/amigo2007a.pdf
 
A subsequent paper identified a use case where **B-Cubed** fared poorly: unbalanced datasets where one cluster dominates: 
 
[3] _Adapted B-CUBED Metrics to Unbalanced Datasets_ by Jose G. Moreno and Gaël Dias, 
    both of Normandie University in France. 
 
This third paper proposed a refined version of **B-Cubed**, but the added complexity adds significantly to processing time, so those refinements are not employed here. The definition of the algorithm used here is taken from section 2.1 of this 
last paper, where it combines the Precision and Recall values into a single number using the **F-measure** formula (a harmonic average). (The refined version is in section 2.2.)

```
    𝔽 = F-measure (final similarity measure)
    ℙ = Precision (a measure of homogeneity)
    ℝ = Recall (a measure of completeness)
    α = Weighting factor (defaults to 0.5)
    ℕ = Number of points
    k = Number of categories (varies between the π and π* Clusterings)
    i = category index
   πᵢ = cluster solution for the ith category
   π*ᵢ= gold standard for the ith category
   g₀ = tests whether two items share the same category in the clustering
   g*₀= tests whether two items share the same category in the gold standard

       𝟙       α     𝟙 - α
     ━━━━━ ═ ━━━━━ + ━━━━━
      𝔽       ℙ       ℝ
       b³      b³      b³

                      k
      ℙ         𝟙    ⎲     𝟙     ⎲   ⎲    
       b³  ═   ━━━   ⎳   ━━━━━   ⎳   ⎳   g*₀(xⱼ,xₗ)
                ℕ    i=1   |πᵢ|   xⱼ∈πᵢ xₗ∈πᵢ

                      k
      ℝ         𝟙    ⎲     𝟙     ⎲    ⎲    
       b³  ═   ━━━   ⎳   ━━━━━   ⎳    ⎳   g₀(xⱼ,xₗ)
                ℕ    i=1   |π*ᵢ|  xⱼ∈π*ᵢ xₗ∈π*ᵢ

               (  𝟙 ⟺ ∃l:xᵢ∈πₗ ∧ xⱼ∈πₗ
  g₀(xᵢ,xⱼ)  ═ <
               (  𝟘, otherwise


               (  𝟙 ⟺ ∃l:xᵢ∈π*ₗ ∧ xⱼ∈π*ₗ
  g*₀(xᵢ,xⱼ) ═ <
               (  𝟘, otherwise

```

If seeing the triply-nested summation symbols makes you balk and say, "that will take forever to run", that was my reaction as well. The inner two loops perform a sort of second moment computation, trying to see what proportion of items that share the same cluster according to one scheme share the matching clusters in the other scheme, and vice versa. Here is an example.

```
            1 2 3 4
            A A B B
          ┍━━━━━━━━━┓
       1 A│ ✓✓     │
       2 A│ ✓✓     │
       3 B│     ✓✓ │
       4 B│     ✓✓ │
          ┗━━━━━━━━━┛
```
In this example:

  - The first clustering has items 1,2,3,4 all in the same cluster.
  - The second clustering puts 1&2 in cluster A and 3&4 in cluster B.
  - The grid shows which cluster in the second clustering each pont goes, and
    pairs of points that continue to be clustered together are marked with checks (✓).

Measuring the effect of the second clustering on the first clustering's items consists in counting the checkmarks in the grid. The full B-Cubed measure normalizes the measure for each cluster by its cluster size.

The optimization that I found was that if two items end up in the same cluster, there will be four check marks, but if three match, there will be nine checks, etc. Thus if you had a list of all the categories in the second clustering that points map into and a count of how many items end up in each category, then you merely need to sum the squares of those counts.

I figured out another trick - you could do this in a single pass! Keeping a running sum of the squares of counts in each category, if you see a count was one and increase it to two, then you add the difference between one and four, which is three. If going from two to three, the increase in the sum of squares is 9 - 4 = 5. The difference is always `2 * previous_count + 1`.

Anyway, that is how I reduced a quadratic algorithm to a linear one.

## Coming Soon...

The next functionality that will be implemented is **Linkage analysis**.


