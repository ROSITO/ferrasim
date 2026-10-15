//! Portable CLI parsing and user-facing reporting; physics lives in the libraries.
use ferrasim_io::{RunOptions, run_simulation_file, validate_simulation_file};
use std::{ffi::OsString, path::PathBuf, process::ExitCode};
const HELP: &str = "FerraSim — linear static TET4 simulation\n\
Usage:\n  ferrasim validate <model.json>\n  ferrasim solve <model.json> --output <new-directory> [--force-scale F] [--length-scale L] [--origin X Y Z]\n\
Options may precede the model path. Use -- before a path beginning with '-'.\n\
Output parent must exist; existing destinations are never overwritten.\n\
Exit codes: 0 success; 2 usage; 3 invalid input/preparation; 4 numerical failure; 5 output failure.\n\
Validation does not establish solvability or independent scientific qualification.";
struct Request {
    model: PathBuf,
    output: Option<PathBuf>,
    options: RunOptions,
}
fn number(value: OsString) -> Result<f64, String> {
    let value = value
        .to_str()
        .ok_or("numeric options must be UTF-8")?
        .parse::<f64>()
        .map_err(|_| "invalid numeric option")?;
    if !value.is_finite() {
        return Err("numeric options must be finite".into());
    }
    Ok(value)
}
fn parse(args: Vec<OsString>, solve: bool) -> Result<Request, String> {
    let mut args = args.into_iter();
    let mut model = None;
    let mut output = None;
    let mut options = RunOptions::default();
    let mut origin_seen = false;
    let mut positional = false;
    while let Some(arg) = args.next() {
        if !positional && arg == "--" {
            positional = true;
            continue;
        }
        if !positional && arg == "--output" {
            if !solve || output.is_some() {
                return Err("unexpected or repeated --output".into());
            }
            output = Some(PathBuf::from(
                args.next().ok_or("--output requires a directory")?,
            ));
            continue;
        }
        if !positional && (arg == "--force-scale" || arg == "--length-scale") {
            if !solve {
                return Err("scale options are only valid for solve".into());
            }
            let target = if arg == "--force-scale" {
                &mut options.force_scale
            } else {
                &mut options.length_scale
            };
            if target.is_some() {
                return Err("repeated scale option".into());
            }
            let value = number(args.next().ok_or("scale option requires a number")?)?;
            if value <= 0.0 {
                return Err("scales must be positive".into());
            }
            *target = Some(value);
            continue;
        }
        if !positional && arg == "--origin" {
            if !solve || origin_seen {
                return Err("unexpected or repeated --origin".into());
            }
            origin_seen = true;
            for x in &mut options.origin {
                *x = number(args.next().ok_or("--origin requires three numbers")?)?;
            }
            continue;
        }
        if !positional && arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option {}", arg.to_string_lossy()));
        }
        if model.replace(PathBuf::from(arg)).is_some() {
            return Err("exactly one model path is required".into());
        }
    }
    if output.as_ref().is_some_and(|p| p.as_os_str().is_empty())
        || model
            .as_ref()
            .is_some_and(|p: &PathBuf| p.as_os_str().is_empty())
    {
        return Err("paths must not be empty".into());
    }
    if solve && output.is_none() {
        return Err("solve requires --output <new-directory>".into());
    }
    Ok(Request {
        model: model.ok_or("missing model path")?,
        output,
        options,
    })
}
fn main() -> ExitCode {
    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || matches!(args.as_slice(),[x] if x=="--help"||x=="-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if matches!(args.as_slice(),[x] if x=="--version"||x=="-V") {
        println!("ferrasim {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let command = args.remove(0);
    let solve = command == "solve";
    if !solve && command != "validate" {
        eprintln!("Unknown command: {}\n{HELP}", command.to_string_lossy());
        return ExitCode::from(2);
    }
    if matches!(args.as_slice(),[x] if x=="--help"||x=="-h") {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    let request = match parse(args, solve) {
        Ok(r) => r,
        Err(error) => {
            eprintln!("Usage error: {error}\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let outcome = if solve {
        run_simulation_file(&request.model,request.output.as_ref().expect("parsed solve output"),&request.options).map(|run|{
            for(id,w)in run.results.material_warnings(){eprintln!("Material {}: {w:?}",id.0);}
            println!("Complete: {} ({} files); iterations={}; residual={:e}; force balance={:e}; moment balance={:e}",run.exported.directory.display(),run.exported.files.len(),run.results.solver().iterations,run.results.solver().residual_norm,run.results.balance().relative_force_error,run.results.balance().relative_moment_error);
        })
    } else {
        validate_simulation_file(&request.model).map(|summary| {
            for warning in summary.warnings {
                eprintln!("Material {warning}");
            }
            println!(
                "Validated: {} nodes, {} TET4. Solvability is not established.",
                summary.nodes, summary.elements
            );
        })
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.diagnostic());
            ExitCode::from(error.exit_code())
        }
    }
}
