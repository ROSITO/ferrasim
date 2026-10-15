//! Solve the model and export selected formats into a new directory.
use ferrasim_fem::{BalanceScales, solve_static_results};
use ferrasim_io::{ResultProvenance, export_results, load_meshed_model};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: export_model <model.json> <new-directory> <force-scale> <length-scale>")?;
    let directory = args.next().ok_or("missing output directory")?;
    let force = args.next().ok_or("missing force scale")?;
    let length = args.next().ok_or("missing length scale")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let scales = BalanceScales::new(
        force.to_str().ok_or("invalid force scale")?.parse()?,
        length.to_str().ok_or("invalid length scale")?.parse()?,
    )?;
    let loaded = load_meshed_model(&path)?;
    let results = solve_static_results(&loaded.mesh, &loaded.model.simulation, [0.0; 3], scales)?;
    let source = ResultProvenance {
        model_source: path.to_string_lossy().into_owned(),
        mesh_source: loaded.model.simulation.mesh.path.clone(),
        declared_code_revision: None,
    };
    let exported = export_results(&results, &source, &directory)?;
    println!(
        "Complete export: {} ({:?})",
        exported.directory.display(),
        exported.files
    );
    Ok(())
}
