//! Sparse structure, arithmetic failures and storage scaling.
use ferrasim_solvers::SparseMatrix;

#[test]
fn duplicates_are_summed_and_csr_is_sorted_without_explicit_zeros() {
    let matrix = SparseMatrix::from_triplets(
        3,
        [
            (2, 0, 4.0),
            (0, 2, 3.0),
            (2, 0, -1.0),
            (1, 1, 2.0),
            (1, 1, -2.0),
        ],
    )
    .unwrap();
    assert_eq!(matrix.csr().row_offsets(), &[0, 1, 1, 2]);
    assert_eq!(matrix.csr().col_indices(), &[2, 0]);
    assert_eq!(
        matrix.multiply(&[2.0, 9.0, -1.0]).unwrap(),
        [-3.0, 0.0, 6.0]
    );
}

#[test]
fn invalid_entries_vectors_and_arithmetic_are_rejected() {
    for entries in [
        vec![(1, 0, 1.0)],
        vec![(0, 1, 1.0)],
        vec![(0, 0, f64::NAN)],
        vec![(0, 0, f64::INFINITY)],
        vec![(0, 0, f64::MAX), (0, 0, f64::MAX)],
    ] {
        assert!(SparseMatrix::from_triplets(1, entries).is_err());
    }
    let m = SparseMatrix::from_triplets(1, [(0, 0, f64::MAX)]).unwrap();
    assert!(m.multiply(&[]).is_err());
    assert!(m.multiply(&[f64::NAN]).is_err());
    assert!(m.multiply(&[2.0]).is_err());
}

#[test]
fn empty_and_large_diagonal_matrices_need_only_sparse_storage() {
    let empty = SparseMatrix::from_triplets(0, []).unwrap();
    assert_eq!(empty.multiply(&[]).unwrap(), Vec::<f64>::new());
    let n = 100_000;
    let m = SparseMatrix::from_triplets(n, (0..n).map(|i| (i, i, 2.0))).unwrap();
    assert_eq!(m.csr().nnz(), n);
    assert_eq!(m.csr().row_offsets().len(), n + 1);
    assert!(m.multiply(&vec![1.0; n]).unwrap().iter().all(|x| *x == 2.0));
}
