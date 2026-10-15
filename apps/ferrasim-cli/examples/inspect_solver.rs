//! Solve an explicit inspection force scenario in place of integrated model forces.
use ferrasim_fem::AssembledSystem;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: inspect_solver <model.json>")?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)?;
    let mut force = vec![0.0; system.dofs().len()];
    let dof = (0..force.len())
        .find(|i| !system.prescriptions().iter().any(|p| p.dof == *i))
        .ok_or("inspection requires a free DOF")?;
    force[dof] = 1.0;
    let reduced = system.reduce(&force)?;
    let report = reduced.solve(&loaded.model.simulation.solver_settings)?;
    let full = reduced.reconstruct(&report.solution)?;
    let reactions = reduced.reactions(&report.solution)?;
    // Output only after all computations succeeded; never write result files here.
    println!(
        "Inspection: unit force at internal DOF {dof}; integrated JSON forces are overridden in this inspection."
    );
    for (material, warning) in system.material_warnings() {
        println!("Material {material:?}: {warning:?}");
    }
    println!(
        "{:?}; iterations={}; residual={:e}; threshold={:e}",
        report.status, report.iterations, report.residual_norm, report.threshold
    );
    println!("Displacements: {full:?}; reactions: {reactions:?}");
    Ok(())
}
