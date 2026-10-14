//! Demonstrate S01 model loading without claiming mesh or solver validation.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = std::env::args_os()
        .nth(1)
        .ok_or("usage: inspect_model <model.json>")?;
    let loaded = ferrasim_io::load_model(source)?;
    println!(
        "Model contract valid (schema {}).",
        loaded.simulation.schema_version
    );
    println!("Mesh location: {}", loaded.mesh_path.display());
    println!("Mesh import and simulation have not been performed.");
    Ok(())
}
