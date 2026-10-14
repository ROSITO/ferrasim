//! Strict versioned JSON loading and portable model-relative mesh paths.

use std::path::{Path, PathBuf};

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
