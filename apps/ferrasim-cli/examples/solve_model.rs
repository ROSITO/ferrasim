//! Solve model loads and inspect balances, with explicit force/length scales.
use ferrasim_fem::{AssembledSystem, BalanceScales};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: solve_model <model.json> <force-scale> <length-scale>")?;
    let force = args.next().ok_or("missing force scale")?;
    let length = args.next().ok_or("missing length scale")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let scales = BalanceScales::new(
        force.to_str().ok_or("invalid force scale")?.parse()?,
        length.to_str().ok_or("invalid length scale")?.parse()?,
    )?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)?;
    let result =
        system.solve_model_loads(&loaded.model.simulation.solver_settings, [0.0; 3], scales)?;
    // Nothing is printed or exported as successful before all computations finish.
    for (material, warning) in system.material_warnings() {
        println!("Material {material:?}: {warning:?}");
    }
    println!(
        "Model loads solved: {:?}; iterations={}; residual={:e}",
        result.solver.status, result.solver.iterations, result.solver.residual_norm
    );
    println!(
        "Balance origin={:?}; force scale={}; length scale={}",
        result.balance.origin,
        scales.force(),
        scales.length()
    );
    println!(
        "Relative force={:e}; moment={:e}; free residual={:e}; prescription={:e}; energy={:e}",
        result.balance.relative_force_error,
        result.balance.relative_moment_error,
        result.balance.relative_free_residual,
        result.balance.relative_prescription_error,
        result.balance.relative_energy_error
    );
    println!(
        "Elastic energy={:e}; applied work={:e}; prescribed reaction work={:e}",
        result.balance.strain_energy, result.balance.load_work, result.balance.prescription_work
    );
    println!(
        "Displacements: {:?}; reactions: {:?}",
        result.displacements, result.reactions
    );
    Ok(())
}
