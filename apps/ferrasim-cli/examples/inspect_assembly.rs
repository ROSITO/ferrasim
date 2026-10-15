//! Inspect global stiffness and kinematic elimination; no loads or solve at S04.
use ferrasim_fem::AssembledSystem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: inspect_assembly <model.json>")?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)?;
    for (material, warning) in system.material_warnings() {
        println!("Material {material:?}: {warning:?}");
    }
    // This explicit zero vector is an inspection scenario, not the JSON loads.
    let reduced = system.reduce(&vec![0.0; system.dofs().len()])?;
    println!(
        "Global DOFs: {}; stored stiffness entries: {}; prescribed: {}; free: {}",
        system.dofs().len(),
        system.stiffness().csr().nnz(),
        system.prescriptions().len(),
        reduced.free_dofs().len()
    );
    println!(
        "Reduced stiffness entries: {}; RHS length: {}",
        reduced.stiffness().csr().nnz(),
        reduced.rhs().len()
    );
    println!(
        "Inspection with an explicit zero force vector. JSON loads are not integrated; no linear solve performed."
    );
    Ok(())
}
