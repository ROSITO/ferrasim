use crate::{
    ExportedResults, MeshIoError, MeshedModelError, ModelIoError, MshError, ResultExportError,
    ResultProvenance, export_results, load_meshed_model,
};
use ferrasim_fem::{
    AssembledSystem, BalanceScales, FemError, SimulationResults, solve_static_results,
};
use ferrasim_mesh::ModelMeshError;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Options shared by the independent Rust API and CLI. No numerical solver settings are overridden.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Optional coherent force scale; None derives it from loads/prescribed displacements.
    pub force_scale: Option<f64>,
    /// Optional coherent length scale; None uses the bounding-box diagonal.
    pub length_scale: Option<f64>,
    /// Global moment origin; default is (0,0,0).
    pub origin: [f64; 3],
}
/// Validation counts and advisories; no linear solve or output publication is performed.
#[derive(Debug)]
pub struct ValidationSummary {
    /// Imported node count.
    pub nodes: usize,
    /// Imported TET4 count.
    pub elements: usize,
    /// Material advisories, for user-visible diagnostics.
    pub warnings: Vec<String>,
}
/// Completed model-file calculation; results remain usable independently of the CLI.
#[derive(Debug)]
pub struct SimulationRun {
    /// Immutable fields and numerical reports.
    pub results: SimulationResults,
    /// Published files and output directory.
    pub exported: ExportedResults,
}
/// Stage-specific workflow errors, with no successful run payload.
#[derive(Debug, Error)]
pub enum SimulationRunError {
    /// Invalid API option, before any output operation.
    #[error("invalid run option: {0}")]
    Options(&'static str),
    /// Model, mesh or binding error with its input location.
    #[error("invalid input {path}: {source}")]
    Input {
        /// Requested source model.
        path: PathBuf,
        /// Structured input error.
        #[source]
        source: MeshedModelError,
    },
    /// Invalid material/assembly/load range before solve.
    #[error("cannot prepare simulation: {0}")]
    Preparation(#[source] FemError),
    /// Solve or field recovery failed.
    #[error("simulation failed: {0}")]
    Numerical(#[source] FemError),
    /// Output destination or publication failure.
    #[error(transparent)]
    Output(#[from] ResultExportError),
}
impl SimulationRunError {
    /// Stable CLI mapping: options=2, input/preparation=3, numerical=4, output=5.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Options(_) => 2,
            Self::Input { .. } | Self::Preparation(_) => 3,
            Self::Numerical(_) => 4,
            Self::Output(_) => 5,
        }
    }
    /// Human-readable context, with field/entity issues when available.
    pub fn diagnostic(&self) -> String {
        let mut text = self.to_string();
        if let Self::Input { source, .. } = self {
            let model_issues = match source {
                MeshedModelError::Model(ModelIoError::Validation(e)) => Some(&e.issues),
                MeshedModelError::Binding(ModelMeshError::Model(e)) => Some(&e.issues),
                _ => None,
            };
            if let Some(issues) = model_issues {
                for i in issues {
                    text.push_str(&format!("\n  {}: {} ({:?})", i.path, i.message, i.code));
                }
            }
            if let MeshedModelError::Binding(ModelMeshError::Binding(e)) = source {
                for i in &e.issues {
                    text.push_str(&format!(
                        "\n  {}: {} (node={:?}, element={:?})",
                        i.path, i.message, i.node, i.element
                    ));
                }
            }
            if let MeshedModelError::Mesh(MeshIoError::Import {
                source: MshError::Mesh(e),
                ..
            }) = source
            {
                for i in &e.issues {
                    text.push_str(&format!(
                        "\n  {} (node={:?}, element={:?})",
                        i.message, i.node, i.element
                    ));
                }
            }
        }
        text
    }
}
fn load(path: &Path) -> Result<crate::MeshedSimulation, SimulationRunError> {
    load_meshed_model(path).map_err(|source| SimulationRunError::Input {
        path: path.to_owned(),
        source,
    })
}
/// Validate JSON, imported geometry, bindings, constitutive operators and integrated loads.
/// Does not establish uniqueness, convergence or scientific qualification; creates no output.
pub fn validate_simulation_file(
    path: impl AsRef<Path>,
) -> Result<ValidationSummary, SimulationRunError> {
    let loaded = load(path.as_ref())?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)
        .map_err(SimulationRunError::Preparation)?;
    Ok(ValidationSummary {
        nodes: system.dofs().nodes().len(),
        elements: loaded.mesh.mesh().tetrahedra.len(),
        warnings: system
            .material_warnings()
            .iter()
            .map(|(id, w)| format!("{}: {w:?}", id.0))
            .collect(),
    })
}
fn scales(system: &AssembledSystem, options: &RunOptions) -> Result<BalanceScales, FemError> {
    let force = match options.force_scale {
        Some(f) => f,
        None => {
            let applied = system
                .forces()
                .chunks_exact(3)
                .map(|f| f[0].hypot(f[1]).hypot(f[2]))
                .sum::<f64>();
            let displacement = system
                .prescriptions()
                .iter()
                .map(|p| p.value.abs())
                .fold(0.0_f64, f64::max);
            let k = system
                .stiffness()
                .csr()
                .values()
                .iter()
                .map(|k| k.abs())
                .fold(0.0_f64, f64::max);
            let prescribed = k * displacement;
            if !applied.is_finite() || !prescribed.is_finite() {
                return Err(FemError::NumericRange(
                    "automatic force scale overflow; supply an explicit scale",
                ));
            }
            let scale = applied.max(prescribed);
            if scale > 0.0 { scale } else { 1.0 }
        }
    };
    let length = match options.length_scale {
        Some(l) => l,
        None => {
            let mut min = [f64::INFINITY; 3];
            let mut max = [f64::NEG_INFINITY; 3];
            for point in system.coordinates() {
                for i in 0..3 {
                    min[i] = min[i].min(point[i]);
                    max[i] = max[i].max(point[i]);
                }
            }
            (max[0] - min[0])
                .hypot(max[1] - min[1])
                .hypot(max[2] - min[2])
        }
    };
    BalanceScales::new(force, length)
}
fn output_error(path: &Path, source: std::io::Error) -> SimulationRunError {
    ResultExportError::Io {
        path: path.to_owned(),
        source,
    }
    .into()
}
fn check_destination(path: &Path) -> Result<(), SimulationRunError> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => {
            return Err(output_error(
                path,
                std::io::Error::from(std::io::ErrorKind::AlreadyExists),
            ));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(output_error(path, e)),
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let metadata = std::fs::metadata(parent).map_err(|e| output_error(parent, e))?;
    if !metadata.is_dir() {
        return Err(output_error(
            parent,
            std::io::Error::from(std::io::ErrorKind::NotADirectory),
        ));
    }
    Ok(())
}
/// Load, solve and publish selected formats, without depending on any CLI package.
/// Output parent must exist; existing destinations (including symlinks) are refused.
/// Mesh paths resolve relative to the model; output paths resolve against the caller's cwd.
pub fn run_simulation_file(
    model_path: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: &RunOptions,
) -> Result<SimulationRun, SimulationRunError> {
    for scale in [options.force_scale, options.length_scale]
        .into_iter()
        .flatten()
    {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(SimulationRunError::Options(
                "scales must be finite and positive",
            ));
        }
    }
    if options.origin.iter().any(|x| !x.is_finite()) {
        return Err(SimulationRunError::Options("origin must be finite"));
    }
    let loaded = load(model_path.as_ref())?;
    check_destination(output.as_ref())?;
    let system = AssembledSystem::assemble(&loaded.mesh, &loaded.model.simulation)
        .map_err(SimulationRunError::Preparation)?;
    let scales = scales(&system, options).map_err(SimulationRunError::Preparation)?;
    drop(system);
    let results = solve_static_results(
        &loaded.mesh,
        &loaded.model.simulation,
        options.origin,
        scales,
    )
    .map_err(SimulationRunError::Numerical)?;
    let provenance = ResultProvenance {
        model_source: loaded.model.source_path.to_string_lossy().into_owned(),
        mesh_source: loaded.model.mesh_path.to_string_lossy().into_owned(),
        declared_code_revision: None,
    };
    let exported = export_results(&results, &provenance, output)?;
    Ok(SimulationRun { results, exported })
}
