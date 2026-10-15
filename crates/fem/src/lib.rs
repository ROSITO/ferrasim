//! Small-strain isotropic elasticity and constant-strain TET4 element operators.
//!
//! Sparse assembly and exact elimination are verified on small reference patches.
//! Numerical verification does not constitute independent scientific qualification.

mod assembly;
mod material;
mod tet4;

pub use assembly::{AssembledSystem, DofMap, PrescribedDof, ReducedSystem};
pub use material::{IsotropicElastic, MaterialWarning, NEAR_INCOMPRESSIBLE_THRESHOLD};
pub use tet4::{ElementResponse, Tet4Element};

use ferrasim_mesh::{ElementId, MeshErrors};
use nalgebra::{SMatrix, SVector};
use thiserror::Error;

/// Stress or engineering-strain vector in order xx, yy, zz, xy, yz, xz.
pub type VoigtVector = SVector<f64, 6>;
/// Constitutive tensor using engineering shear strains.
pub type ElasticityMatrix = SMatrix<f64, 6, 6>;
/// Local displacements ordered node-by-node, three Cartesian components each.
pub type ElementDisplacements = SVector<f64, 12>;
/// Constant strain-displacement operator for a TET4.
pub type StrainDisplacementMatrix = SMatrix<f64, 6, 12>;
/// Element stiffness in original local node ordering.
pub type ElementStiffness = SMatrix<f64, 12, 12>;

/// Failures constructing or evaluating a constitutive law or element.
#[derive(Debug, Error)]
pub enum FemError {
    /// Model or mesh binding does not satisfy the simulation contract.
    #[error(transparent)]
    Binding(#[from] ferrasim_mesh::ModelMeshError),
    /// Invalid assembly, constraint, or vector contract.
    #[error("FEM assembly failure: {0}")]
    Assembly(&'static str),
    /// Sparse storage or arithmetic failure.
    #[error(transparent)]
    Sparse(#[from] ferrasim_solvers::SparseError),
    /// Invalid physical material parameters.
    #[error("invalid isotropic material: {0}")]
    InvalidMaterial(&'static str),
    /// Nonfinite input, overflow, underflow or a numerically unusable operator.
    #[error("FEM numeric range failure: {0}")]
    NumericRange(&'static str),
    /// Source mesh fails the S02 geometric checks.
    #[error(transparent)]
    Mesh(#[from] MeshErrors),
    /// Requested external element ID is not present.
    #[error("tetrahedron {0:?} is absent from the mesh")]
    MissingElement(ElementId),
}
