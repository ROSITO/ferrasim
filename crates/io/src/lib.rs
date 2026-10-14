//! Strict versioned JSON loading and portable model-relative mesh paths.

mod gmsh;

pub use gmsh::{MshCode, MshError, parse_msh22, parse_msh22_bytes};

use std::path::{Path, PathBuf};

use ferrasim_mesh::{ModelMeshError, ValidatedMesh};
use ferrasim_model::{SCHEMA_VERSION, Simulation, ValidationErrors};
use serde::Deserialize;
use thiserror::Error;

/// Structured failures from parsing, reading or validating a model.
#[derive(Debug, Error)]
pub enum ModelIoError {
    /// Filesystem failure; includes the requested source path.
    #[error("cannot read model {path}: {source}")]
    Read {
        /// Model source path.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// Malformed JSON, unsupported shape, missing fields or unknown fields.
    #[error("invalid model JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Schema version is explicitly unsupported.
    #[error("unsupported schema version {found:?}; expected {SCHEMA_VERSION}")]
    UnsupportedVersion {
        /// Unsupported version found in the header.
        found: String,
    },
    /// One or more semantic model issues.
    #[error(transparent)]
    Validation(#[from] ValidationErrors),
}

#[derive(Deserialize)]
struct VersionHeader {
    schema_version: String,
}

/// Parse and validate a JSON model without opening its mesh.
///
/// Unknown fields and repeated known keys are errors; unknown versions are
/// rejected before applying the current schema. No defaults alter the physics.
pub fn parse_model_json(json: &str) -> Result<Simulation, ModelIoError> {
    let header: VersionHeader = serde_json::from_str(json)?;
    if header.schema_version != SCHEMA_VERSION {
        return Err(ModelIoError::UnsupportedVersion {
            found: header.schema_version,
        });
    }
    let simulation: Simulation = serde_json::from_str(json)?;
    simulation.validate()?;
    Ok(simulation)
}

/// Validate before serializing, so nonfinite API values cannot become JSON nulls.
pub fn model_to_json(simulation: &Simulation) -> Result<String, ModelIoError> {
    simulation.validate()?;
    Ok(serde_json::to_string_pretty(simulation)?)
}

/// A validated model and its absolute source-relative mesh location.
#[derive(Debug)]
pub struct LoadedSimulation {
    /// Model-level validated description, without a mesh solvability guarantee.
    pub simulation: Simulation,
    /// Canonical source file path.
    pub source_path: PathBuf,
    /// Absolute mesh location; existence and topology are checked during import.
    pub mesh_path: PathBuf,
}

/// Read a model and resolve its mesh path relative to the source file.
///
/// The process working directory does not affect mesh resolution. The mesh is
/// not read here; paths containing `..` are allowed and preserved lexically.
pub fn load_model(path: impl AsRef<Path>) -> Result<LoadedSimulation, ModelIoError> {
    let requested = path.as_ref();
    let read_error = |source| ModelIoError::Read {
        path: requested.to_path_buf(),
        source,
    };
    let source_path = requested.canonicalize().map_err(read_error)?;
    let json = std::fs::read_to_string(&source_path).map_err(read_error)?;
    let simulation = parse_model_json(&json)?;
    let mut mesh_path = source_path.clone();
    mesh_path.pop();
    for component in simulation.mesh.path.split('/') {
        mesh_path.push(component);
    }
    Ok(LoadedSimulation {
        simulation,
        source_path,
        mesh_path,
    })
}

/// Filesystem or import failure, retaining the mesh source path.
#[derive(Debug, Error)]
pub enum MeshIoError {
    /// Cannot read the requested mesh.
    #[error("cannot read mesh {path}: {source}")]
    Read {
        /// Requested file.
        path: PathBuf,
        /// Filesystem cause.
        #[source]
        source: std::io::Error,
    },
    /// Invalid or unsupported mesh.
    #[error("cannot import mesh {path}: {source}")]
    Import {
        /// Requested file.
        path: PathBuf,
        /// Structured format or geometry failure.
        #[source]
        source: MshError,
    },
}

/// Read and validate an MSH 2.2 ASCII mesh, detecting binary headers first.
pub fn load_msh22(path: impl AsRef<Path>) -> Result<ValidatedMesh, MeshIoError> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|source| MeshIoError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    parse_msh22_bytes(&bytes).map_err(|source| MeshIoError::Import {
        path: path.to_path_buf(),
        source,
    })
}

/// Failure of model loading, mesh import or their binding.
#[derive(Debug, Error)]
pub enum MeshedModelError {
    /// Invalid source model.
    #[error(transparent)]
    Model(#[from] ModelIoError),
    /// Invalid source mesh.
    #[error(transparent)]
    Mesh(#[from] MeshIoError),
    /// Model selections are incompatible with the actual mesh.
    #[error(transparent)]
    Binding(#[from] ModelMeshError),
}

/// Validated model and mesh, without an assembled or solved mechanical system.
#[derive(Debug)]
pub struct MeshedSimulation {
    /// Model and resolved source locations.
    pub model: LoadedSimulation,
    /// Immutable validated mesh.
    pub mesh: ValidatedMesh,
}

/// Load a model, import its mesh and verify all selections and material coverage.
pub fn load_meshed_model(path: impl AsRef<Path>) -> Result<MeshedSimulation, MeshedModelError> {
    let model = load_model(path)?;
    let mesh = load_msh22(&model.mesh_path)?;
    mesh.validate_model(&model.simulation)?;
    Ok(MeshedSimulation { model, mesh })
}
