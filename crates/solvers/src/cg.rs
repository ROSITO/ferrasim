use crate::SparseMatrix;
use ferrasim_model::{Preconditioner, SolverMethod, SolverSettings};
use thiserror::Error;

/// Maximum relative mismatch of a symmetric pair accepted by CG.
pub const CG_SYMMETRY_TOLERANCE: f64 = 1e-12;

/// Explicit terminal state of a CG attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStatus {
    /// The freshly computed residual satisfies the requested criterion.
    Converged,
    /// The budget expired before verified convergence.
    IterationLimit,
    /// Settings or the right-hand side are invalid.
    InvalidInput,
    /// Symmetry or a positive usable Jacobi diagonal is missing.
    InvalidMatrix,
    /// Arithmetic or a positive-curvature condition failed.
    NumericalBreakdown,
}

/// Stable classification of a failed solve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveFailureReason {
    /// Nonfinite/negative tolerances, both zero, or zero iteration budget.
    InvalidSettings,
    /// Invalid vector length or a nonfinite vector entry.
    InvalidRightHandSide,
    /// A symmetric matrix pair differs beyond the published relative tolerance.
    NotSymmetric,
    /// A diagonal entry is missing, nonpositive, or has no finite inverse.
    InvalidJacobiDiagonal,
    /// Nonfinite, overflowing or underflowed required scalar/vector arithmetic.
    Arithmetic,
    /// Nonpositive p^T A p: CG cannot continue on this direction.
    NonPositiveCurvature,
    /// Nonpositive r^T M^-1 r outside verified convergence.
    NonPositivePreconditionedResidual,
    /// The verified residual still exceeds tolerance at the iteration limit.
    IterationBudget,
}

/// Successful CG result. No instance is returned for a failed solve.
#[derive(Debug, Clone)]
pub struct SolveReport {
    /// Always `Converged` for a report returned by the solver.
    pub status: SolveStatus,
    /// Solution in the matrix's internal degree-of-freedom order.
    pub solution: Vec<f64>,
    /// Number of completed CG updates (zero for an initially accepted solution).
    pub iterations: u64,
    /// Euclidean norm of the freshly computed b-Ax.
    pub residual_norm: f64,
    /// Residual norm divided by norm(b); None for a zero denominator.
    pub relative_residual: Option<f64>,
    /// Requested absolute criterion: atol + rtol * norm(b).
    pub threshold: f64,
}

/// Failed solve, with no solution/result payload that could be mistaken for success.
#[derive(Debug, Error, Clone)]
#[error(
    "CG/Jacobi {status:?}: {reason:?} after {iterations} updates (residual {residual_norm:?}, threshold {threshold:?})"
)]
pub struct SolveError {
    /// Terminal failure state.
    pub status: SolveStatus,
    /// Stable detailed category.
    pub reason: SolveFailureReason,
    /// Completed CG updates.
    pub iterations: u64,
    /// Fresh residual norm when computable, otherwise None.
    pub residual_norm: Option<f64>,
    /// Fresh residual divided by norm(b), if finite and norm(b) is nonzero.
    pub relative_residual: Option<f64>,
    /// Requested threshold when computable.
    pub threshold: Option<f64>,
}

fn norm(values: &[f64]) -> Option<f64> {
    let value = values.iter().fold(0.0_f64, |acc, x| acc.hypot(*x));
    value.is_finite().then_some(value)
}
fn dot(a: &[f64], b: &[f64]) -> Option<f64> {
    let mut dot = crate::AccurateDot::default();
    for (a, b) in a.iter().zip(b) {
        dot.add(*a, *b)?;
    }
    dot.finish()
}
fn residual(matrix: &SparseMatrix, rhs: &[f64], x: &[f64]) -> Option<Vec<f64>> {
    let ax = matrix.multiply(x).ok()?;
    let result: Vec<_> = rhs.iter().zip(ax).map(|(b, ax)| b - ax).collect();
    result.iter().all(|x| x.is_finite()).then_some(result)
}
fn relative(residual: f64, rhs: Option<f64>) -> Option<f64> {
    rhs.filter(|b| *b > 0.0).and_then(|b| {
        let ratio = residual / b;
        ratio.is_finite().then_some(ratio)
    })
}
struct Attempt<'a> {
    matrix: &'a SparseMatrix,
    rhs: &'a [f64],
    rhs_norm: Option<f64>,
    threshold: Option<f64>,
}
impl Attempt<'_> {
    fn fail(
        &self,
        status: SolveStatus,
        reason: SolveFailureReason,
        iterations: u64,
        x: &[f64],
    ) -> SolveError {
        let residual_norm = residual(self.matrix, self.rhs, x).and_then(|r| norm(&r));
        SolveError {
            status,
            reason,
            iterations,
            residual_norm,
            relative_residual: residual_norm.and_then(|r| relative(r, self.rhs_norm)),
            threshold: self.threshold,
        }
    }
    fn success(&self, solution: Vec<f64>, iterations: u64, residual_norm: f64) -> SolveReport {
        SolveReport {
            status: SolveStatus::Converged,
            solution,
            iterations,
            residual_norm,
            relative_residual: relative(residual_norm, self.rhs_norm),
            threshold: self.threshold.expect("validated threshold"),
        }
    }
}

/// Solve a symmetric positive definite system by CG with diagonal Jacobi preconditioning.
///
/// Starts at zero; checks symmetry and positive finite invertible diagonals even
/// when b=0. Verifies b-Ax at every update and before success. These checks do not
/// certify positive definiteness or detect every singular system. Arithmetic outside
/// representable f64 ranges returns an error, without regularization or dense fallback.
/// Empty 0x0 systems are accepted with a zero-update solution if settings are valid.
pub fn solve_cg_jacobi(
    matrix: &SparseMatrix,
    rhs: &[f64],
    settings: &SolverSettings,
) -> Result<SolveReport, SolveError> {
    // Exhaustive dispatch: future model choices must be handled explicitly.
    match (&settings.method, &settings.preconditioner) {
        (SolverMethod::Cg, Preconditioner::Jacobi) => {}
    }
    let mut attempt = Attempt {
        matrix,
        rhs,
        rhs_norm: None,
        threshold: None,
    };
    // Do not allocate a candidate before validating the public vector contract.
    if rhs.len() != matrix.size() || rhs.iter().any(|x| !x.is_finite()) {
        return Err(SolveError {
            status: SolveStatus::InvalidInput,
            reason: SolveFailureReason::InvalidRightHandSide,
            iterations: 0,
            residual_norm: None,
            relative_residual: None,
            threshold: None,
        });
    }
    let mut x = vec![0.0; matrix.size()];
    let mut x_correction = vec![0.0; matrix.size()];
    attempt.rhs_norm = norm(rhs);
    if !(settings.rtol.is_finite() && settings.atol.is_finite())
        || settings.rtol < 0.0
        || settings.atol < 0.0
        || (settings.rtol == 0.0 && settings.atol == 0.0)
        || settings.max_iterations == 0
    {
        return Err(attempt.fail(
            SolveStatus::InvalidInput,
            SolveFailureReason::InvalidSettings,
            0,
            &x,
        ));
    }
    let rhs_norm = attempt.rhs_norm.ok_or_else(|| {
        attempt.fail(
            SolveStatus::NumericalBreakdown,
            SolveFailureReason::Arithmetic,
            0,
            &x,
        )
    })?;
    let threshold = settings.atol + settings.rtol * rhs_norm;
    if !threshold.is_finite() || (threshold == 0.0 && rhs_norm > 0.0) {
        return Err(attempt.fail(
            SolveStatus::NumericalBreakdown,
            SolveFailureReason::Arithmetic,
            0,
            &x,
        ));
    }
    attempt.threshold = Some(threshold);
    let csr = matrix.csr();
    let mut diagonal = vec![0.0; matrix.size()];
    for (i, j, value) in matrix.entries() {
        if i == j {
            diagonal[i] = value;
        }
        let start = csr.row_offsets()[j];
        let end = csr.row_offsets()[j + 1];
        let transpose = csr.col_indices()[start..end]
            .binary_search(&i)
            .ok()
            .map(|k| csr.values()[start + k])
            .unwrap_or(0.0);
        let scale = value.abs().max(transpose.abs());
        if (value / scale - transpose / scale).abs() > CG_SYMMETRY_TOLERANCE {
            return Err(attempt.fail(
                SolveStatus::InvalidMatrix,
                SolveFailureReason::NotSymmetric,
                0,
                &x,
            ));
        }
    }
    if diagonal.iter().any(|d| *d <= 0.0 || !(1.0 / d).is_finite()) {
        return Err(attempt.fail(
            SolveStatus::InvalidMatrix,
            SolveFailureReason::InvalidJacobiDiagonal,
            0,
            &x,
        ));
    }
    if rhs_norm <= threshold {
        return Ok(attempt.success(x, 0, rhs_norm));
    }
    let precondition = |r: &[f64]| -> Option<Vec<f64>> {
        let z: Vec<_> = r.iter().zip(&diagonal).map(|(r, d)| r / d).collect();
        z.iter().all(|v| v.is_finite()).then_some(z)
    };
    let mut r = rhs.to_vec();
    let mut z = precondition(&r).ok_or_else(|| {
        attempt.fail(
            SolveStatus::NumericalBreakdown,
            SolveFailureReason::Arithmetic,
            0,
            &x,
        )
    })?;
    let mut rho = dot(&r, &z).ok_or_else(|| {
        attempt.fail(
            SolveStatus::NumericalBreakdown,
            SolveFailureReason::Arithmetic,
            0,
            &x,
        )
    })?;
    let mut p = z.clone();
    for iteration in 1..=settings.max_iterations {
        if rho <= 0.0 {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::NonPositivePreconditionedResidual,
                iteration - 1,
                &x,
            ));
        }
        let ap = matrix.multiply(&p).map_err(|_| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration - 1,
                &x,
            )
        })?;
        let curvature = dot(&p, &ap).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration - 1,
                &x,
            )
        })?;
        if curvature <= 0.0 {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::NonPositiveCurvature,
                iteration - 1,
                &x,
            ));
        }
        let alpha = rho / curvature;
        if !alpha.is_finite() || alpha == 0.0 {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration - 1,
                &x,
            ));
        }
        for i in 0..x.len() {
            let increment = alpha.mul_add(p[i], -x_correction[i]);
            let next = x[i] + increment;
            x_correction[i] = (next - x[i]) - increment;
            x[i] = next;
            r[i] = (-alpha).mul_add(ap[i], r[i]);
        }
        if x.iter()
            .chain(&r)
            .chain(&x_correction)
            .any(|v| !v.is_finite())
        {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            ));
        }
        let true_r = residual(matrix, rhs, &x).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            )
        })?;
        let true_norm = norm(&true_r).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            )
        })?;
        if true_norm <= threshold {
            return Ok(attempt.success(x, iteration, true_norm));
        }
        if iteration == settings.max_iterations {
            return Err(attempt.fail(
                SolveStatus::IterationLimit,
                SolveFailureReason::IterationBudget,
                iteration,
                &x,
            ));
        }
        // Restart only when the recursive residual would incorrectly claim convergence.
        let restart = norm(&r).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            )
        })? <= threshold;
        if restart {
            // The fresh residual refers to the rounded public iterate. Discard the
            // unrepresented increment when restarting from that iterate.
            x_correction.fill(0.0);
            r = true_r;
        }
        z = precondition(&r).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            )
        })?;
        let next_rho = dot(&r, &z).ok_or_else(|| {
            attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            )
        })?;
        if next_rho <= 0.0 {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::NonPositivePreconditionedResidual,
                iteration,
                &x,
            ));
        }
        let beta = if restart { 0.0 } else { next_rho / rho };
        if !beta.is_finite() || (!restart && beta == 0.0) {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            ));
        }
        for i in 0..p.len() {
            p[i] = if restart { z[i] } else { z[i] + beta * p[i] };
        }
        if p.iter().any(|v| !v.is_finite()) {
            return Err(attempt.fail(
                SolveStatus::NumericalBreakdown,
                SolveFailureReason::Arithmetic,
                iteration,
                &x,
            ));
        }
        rho = next_rho;
    }
    unreachable!("positive validated iteration budget")
}
