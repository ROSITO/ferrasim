//! Small reproducible CG success/failure scenarios; no result files are written.
use ferrasim_model::{Preconditioner, SolverMethod, SolverSettings};
use ferrasim_solvers::{SparseMatrix, solve_cg_jacobi};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scenario = std::env::args().nth(1).unwrap_or("spd".into());
    let (entries, rhs, budget) = match scenario.as_str() {
        "spd" => (
            [(0, 0, 4.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)],
            [1.0, 2.0],
            100,
        ),
        "limit" => (
            [(0, 0, 4.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)],
            [1.0, 2.0],
            1,
        ),
        "singular" => (
            [(0, 0, 1.0), (0, 1, -1.0), (1, 0, -1.0), (1, 1, 1.0)],
            [1.0, 1.0],
            100,
        ),
        _ => return Err("usage: cg_reference [spd|singular|limit]".into()),
    };
    let matrix = SparseMatrix::from_triplets(2, entries)?;
    let settings = SolverSettings {
        method: SolverMethod::Cg,
        preconditioner: Preconditioner::Jacobi,
        rtol: 1e-12,
        atol: 1e-14,
        max_iterations: budget,
    };
    let report = solve_cg_jacobi(&matrix, &rhs, &settings)?;
    println!(
        "{:?}; iterations={}; residual={:e}; solution={:?}",
        report.status, report.iterations, report.residual_norm, report.solution
    );
    Ok(())
}
