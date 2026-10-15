//! Complete calculation through the Rust API, without the CLI package.
use ferrasim_io::{RunOptions, run_simulation_file};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let model = args
        .next()
        .ok_or("usage: run_simulation <model.json> <new-directory>")?;
    let output = args.next().ok_or("missing new output directory")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let run = run_simulation_file(model, output, &RunOptions::default())?;
    println!(
        "Complete API calculation: {}; {} nodes, {} TET4",
        run.exported.directory.display(),
        run.results.nodes().len(),
        run.results.elements().len()
    );
    Ok(())
}
