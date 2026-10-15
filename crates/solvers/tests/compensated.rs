//! Exact binary cancellation fixtures, independent of FEM and CG recurrence.
use ferrasim_solvers::SparseMatrix;

#[test]
fn retains_small_term_between_large_opposite_terms() {
    let large = 2_f64.powi(54);
    let a = SparseMatrix::from_triplets(3, [(0, 0, large), (0, 1, 1.0), (0, 2, -large)]).unwrap();
    assert_eq!(a.multiply(&[1.0; 3]).unwrap(), [1.0, 0.0, 0.0]);
}

#[test]
fn retains_product_rounding_after_cancellation() {
    // (1+2^-27)(1-2^-27)-1 = -2^-54 exactly. Rounded products alone lose it.
    let epsilon = 2_f64.powi(-27);
    let a = SparseMatrix::from_triplets(2, [(0, 0, 1.0 + epsilon), (0, 1, -1.0)]).unwrap();
    assert_eq!(
        a.multiply(&[1.0 - epsilon, 1.0]).unwrap()[0],
        -2_f64.powi(-54)
    );
}
