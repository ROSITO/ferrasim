//! S02 demonstration of mesh import and model references, without FEM solve.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: inspect_mesh <model.json>")?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    println!("Model references and mesh integrity checked.");
    println!(
        "{} nodes, {} tetrahedra, {} triangles ({} exterior).",
        loaded.mesh.mesh().nodes.len(),
        loaded.mesh.mesh().tetrahedra.len(),
        loaded.mesh.mesh().triangles.len(),
        loaded.mesh.boundary_faces().len()
    );
    let reversed = loaded
        .mesh
        .boundary_faces()
        .values()
        .filter(|face| !face.source_is_outward)
        .count();
    println!(
        "{reversed} source boundary triangles have inward winding; outward metadata comes from volumes."
    );
    println!("No mechanical system has been assembled or solved.");
    Ok(())
}
