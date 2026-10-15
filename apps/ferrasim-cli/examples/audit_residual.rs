//! Export a solved reduced system for an external high-precision residual audit.
//! Diagnostic only: not a simulation export and never a qualification report.
use ferrasim_fem::AssembledSystem;
use std::io::{BufWriter, Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: audit_residual <model.json>")?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)?;
    let reduced = system.reduce(system.forces())?;
    let report = reduced.solve(&loaded.model.simulation.solver_settings)?;
    let mut out = BufWriter::new(std::io::stdout().lock());
    writeln!(
        out,
        "{{\"rhs\":{:?},\"x\":{:?},\"residual_norm\":{:?},\"entries\":[",
        reduced.rhs(),
        report.solution,
        report.residual_norm
    )?;
    let mut first = true;
    for (i, j, a) in reduced.stiffness().entries() {
        if !first {
            write!(out, ",")?;
        }
        first = false;
        write!(out, "[{i},{j},{a:?}]")?;
    }
    writeln!(out, "]}}")?;
    Ok(())
}
