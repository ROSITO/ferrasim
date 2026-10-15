//! CG criteria, independent dense comparisons, singularity and failure contracts.
use ferrasim_model::{Preconditioner, SolverMethod, SolverSettings};
use ferrasim_solvers::{SolveFailureReason as Reason, SolveStatus, SparseMatrix, solve_cg_jacobi};
use nalgebra::{DMatrix, DVector};
fn settings() -> SolverSettings {
    SolverSettings {
        method: SolverMethod::Cg,
        preconditioner: Preconditioner::Jacobi,
        rtol: 1e-12,
        atol: 1e-14,
        max_iterations: 1000,
    }
}
fn matrix(values: &[&[f64]]) -> SparseMatrix {
    SparseMatrix::from_triplets(
        values.len(),
        values
            .iter()
            .enumerate()
            .flat_map(|(i, row)| row.iter().enumerate().map(move |(j, v)| (i, j, *v))),
    )
    .unwrap()
}
fn norm(x: &[f64]) -> f64 {
    x.iter().fold(0.0_f64, |n, x| n.hypot(*x))
}
fn verify(a: &SparseMatrix, b: &[f64], options: &SolverSettings, expected: &[f64]) {
    let report = solve_cg_jacobi(a, b, options).unwrap();
    assert_eq!(report.status, SolveStatus::Converged);
    let residual: Vec<_> = a
        .multiply(&report.solution)
        .unwrap()
        .iter()
        .zip(b)
        .map(|(ax, b)| b - ax)
        .collect();
    let measured = norm(&residual);
    assert_eq!(measured, report.residual_norm);
    assert!(measured <= options.atol + options.rtol * norm(b));
    let error: Vec<_> = report
        .solution
        .iter()
        .zip(expected)
        .map(|(x, y)| x - y)
        .collect();
    let error = norm(&error) / norm(expected).max(1.0);
    assert!(error < 1e-10, "solution error {error:e}");
    assert!(report.iterations <= options.max_iterations);
    println!(
        "n={}, iterations={}, displacement={error:e}, residual={measured:e}",
        a.size(),
        report.iterations
    );
}
#[test]
fn coupled_spd_corpus_agrees_with_independent_dense_lu() {
    for n in [2, 5, 17, 40] {
        // Symmetric strictly diagonally dominant positive matrix, not assembled by FEM.
        let mut dense = DMatrix::zeros(n, n);
        for i in 0..n {
            for j in 0..i {
                let value = (((i * 13 + j * 7) % 11) as f64 - 5.0) / 20.0;
                dense[(i, j)] = value;
                dense[(j, i)] = value;
            }
        }
        for i in 0..n {
            dense[(i, i)] = 1.0
                + (0..n)
                    .filter(|j| *j != i)
                    .map(|j| dense[(i, j)].abs())
                    .sum::<f64>();
        }
        let b: Vec<_> = (0..n).map(|i| ((i * 3) % 7) as f64 - 3.0).collect();
        let expected = dense
            .clone()
            .lu()
            .solve(&DVector::from_column_slice(&b))
            .unwrap();
        let a = SparseMatrix::from_triplets(
            n,
            (0..n).flat_map(|i| {
                (0..n).map({
                    let dense = &dense;
                    move |j| (i, j, dense[(i, j)])
                })
            }),
        )
        .unwrap();
        verify(&a, &b, &settings(), expected.as_slice());
    }
}
#[test]
fn jacobi_solves_widely_scaled_diagonal_in_one_update() {
    let a = matrix(&[&[1e-100, 0.0, 0.0], &[0.0, 2.0, 0.0], &[0.0, 0.0, 1e100]]);
    let b = [1.0, -2.0, 3.0];
    let report = solve_cg_jacobi(&a, &b, &settings()).unwrap();
    assert_eq!(report.iterations, 1);
    verify(&a, &b, &settings(), &[1e100, -1.0, 3e-100]);
}
#[test]
fn large_sparse_poisson_system_converges_without_dense_storage() {
    let n = 2000;
    let a = SparseMatrix::from_triplets(
        n,
        (0..n).flat_map(|i| {
            let mut row = vec![(i, i, 3.0)];
            if i > 0 {
                row.push((i, i - 1, -1.0));
            }
            if i + 1 < n {
                row.push((i, i + 1, -1.0));
            }
            row
        }),
    )
    .unwrap();
    let expected: Vec<_> = (0..n).map(|i| ((i % 17) as f64 - 8.0) / 10.0).collect();
    let b = a.multiply(&expected).unwrap();
    assert_eq!(a.csr().nnz(), 3 * n - 2);
    verify(&a, &b, &settings(), &expected);
}
#[test]
fn zero_rhs_and_empty_system_have_defined_zero_update_results() {
    let mut options = settings();
    options.atol = 0.0;
    for a in [
        matrix(&[&[2.0, 1.0], &[1.0, 2.0]]),
        SparseMatrix::from_triplets(0, []).unwrap(),
    ] {
        let report = solve_cg_jacobi(&a, &vec![0.0; a.size()], &options).unwrap();
        assert_eq!(report.iterations, 0);
        assert_eq!(report.residual_norm, 0.0);
        assert_eq!(report.threshold, 0.0);
        assert_eq!(report.relative_residual, None);
    }
}
#[test]
fn absolute_and_relative_criteria_are_applied_as_requested() {
    let a = matrix(&[&[2.0]]);
    let mut options = settings();
    options.rtol = 0.0;
    options.atol = 1e-6;
    let report = solve_cg_jacobi(&a, &[1e-7], &options).unwrap();
    assert_eq!(report.iterations, 0);
    assert_eq!(report.relative_residual, Some(1.0));
    options.atol = 1e-12;
    verify(&a, &[1e-7], &options, &[5e-8]);
    options.atol = 0.0;
    options.rtol = 1e-8;
    verify(&a, &[20.0], &options, &[10.0]);
}
#[test]
fn invalid_settings_and_vectors_are_errors_before_iteration() {
    let a = matrix(&[&[1.0]]);
    for (rtol, atol, budget) in [
        (f64::NAN, 1.0, 1),
        (1.0, f64::INFINITY, 1),
        (-1.0, 1.0, 1),
        (1.0, -1.0, 1),
        (0.0, 0.0, 1),
        (1e-5, 0.0, 0),
    ] {
        let mut options = settings();
        options.rtol = rtol;
        options.atol = atol;
        options.max_iterations = budget;
        let error = solve_cg_jacobi(&a, &[1.0], &options).unwrap_err();
        assert_eq!(error.status, SolveStatus::InvalidInput);
        assert_eq!(error.reason, Reason::InvalidSettings);
        assert_eq!(error.iterations, 0);
    }
    for b in [vec![], vec![1.0, 2.0], vec![f64::NAN], vec![f64::INFINITY]] {
        assert_eq!(
            solve_cg_jacobi(&a, &b, &settings()).unwrap_err().reason,
            Reason::InvalidRightHandSide
        );
    }
}
#[test]
fn missing_nonpositive_and_noninvertible_diagonals_fail_even_with_zero_rhs() {
    for a in [matrix(&[&[0.0]]), matrix(&[&[-1.0]]), matrix(&[&[1e-310]])] {
        let error = solve_cg_jacobi(&a, &[0.0], &settings()).unwrap_err();
        assert_eq!(error.status, SolveStatus::InvalidMatrix);
        assert_eq!(error.reason, Reason::InvalidJacobiDiagonal);
    }
}
#[test]
fn asymmetry_is_rejected_without_silent_symmetrization() {
    for a in [
        matrix(&[&[2.0, 1.0], &[0.0, 2.0]]),
        matrix(&[&[2.0, 1.0], &[1.0001, 2.0]]),
    ] {
        let error = solve_cg_jacobi(&a, &[0.0, 0.0], &settings()).unwrap_err();
        assert_eq!(error.reason, Reason::NotSymmetric);
    }
    let a = matrix(&[&[2.0, 1.0], &[1.0 + 1e-14, 2.0]]);
    assert!(solve_cg_jacobi(&a, &[1.0, 2.0], &settings()).is_ok());
}
#[test]
fn singular_reference_and_negative_curvature_fail_explicitly() {
    // Singular positive-diagonal reference: b lies in its nullspace.
    for (a, b) in [
        (matrix(&[&[1.0, -1.0], &[-1.0, 1.0]]), [1.0, 1.0]),
        (matrix(&[&[1.0, 2.0], &[2.0, 1.0]]), [1.0, -1.0]),
    ] {
        let error = solve_cg_jacobi(&a, &b, &settings()).unwrap_err();
        assert_eq!(error.status, SolveStatus::NumericalBreakdown);
        assert_eq!(error.reason, Reason::NonPositiveCurvature);
        assert_eq!(error.iterations, 0);
        assert!(error.residual_norm.unwrap() > 0.0);
    }
}
#[test]
fn iteration_limit_returns_only_failure_with_fresh_residual() {
    let a = matrix(&[&[4.0, 1.0], &[1.0, 3.0]]);
    let mut options = settings();
    options.max_iterations = 1;
    let b = [1.0, 2.0];
    let error = solve_cg_jacobi(&a, &b, &options).unwrap_err();
    assert_eq!(error.status, SolveStatus::IterationLimit);
    assert_eq!(error.reason, Reason::IterationBudget);
    assert_eq!(error.iterations, 1);
    // Independently compute the first preconditioned update and its true residual.
    let z = [b[0] / 4.0, b[1] / 3.0];
    let az = [4.0 * z[0] + z[1], z[0] + 3.0 * z[1]];
    let alpha = (b[0] * z[0] + b[1] * z[1]) / (z[0] * az[0] + z[1] * az[1]);
    let r = [b[0] - alpha * az[0], b[1] - alpha * az[1]];
    assert!((error.residual_norm.unwrap() - norm(&r)).abs() < 1e-14);
    assert!(error.residual_norm.unwrap() > error.threshold.unwrap());
}
#[test]
fn arithmetic_overflow_and_underflow_are_failures_not_convergence() {
    let a = matrix(&[&[1.0]]);
    let mut options = settings();
    assert_eq!(
        solve_cg_jacobi(&a, &[1e200], &options).unwrap_err().reason,
        Reason::Arithmetic
    );
    options.rtol = f64::MAX;
    options.atol = f64::MAX;
    assert_eq!(
        solve_cg_jacobi(&a, &[2.0], &options).unwrap_err().reason,
        Reason::Arithmetic
    );
    options = settings();
    options.atol = 0.0;
    options.rtol = f64::MIN_POSITIVE;
    assert_eq!(
        solve_cg_jacobi(&a, &[1e-200], &options).unwrap_err().reason,
        Reason::Arithmetic
    );
    options = settings();
    options.atol = 1e-310;
    options.rtol = 0.0;
    let error = solve_cg_jacobi(&a, &[1e-200], &options).unwrap_err();
    assert_eq!(error.status, SolveStatus::NumericalBreakdown);
}

#[test]
fn extremely_strict_tolerance_never_accepts_a_recursive_residual_alone() {
    let a = matrix(&[&[5.1, 0.7, -0.2], &[0.7, 3.3, 0.9], &[-0.2, 0.9, 2.7]]);
    let b = [2.0_f64.sqrt(), 3.0_f64.sqrt(), 5.0_f64.sqrt()];
    let mut options = settings();
    options.atol = 0.0;
    options.rtol = 1e-30;
    options.max_iterations = 50;
    match solve_cg_jacobi(&a, &b, &options) {
        Ok(report) => {
            let ax = a.multiply(&report.solution).unwrap();
            let fresh: Vec<_> = b.iter().zip(ax).map(|(b, ax)| b - ax).collect();
            assert!(norm(&fresh) <= options.rtol * norm(&b));
        }
        Err(error) => {
            assert!(matches!(
                error.status,
                SolveStatus::NumericalBreakdown | SolveStatus::IterationLimit
            ));
            assert!(error.residual_norm.unwrap() > error.threshold.unwrap());
        }
    }
}
