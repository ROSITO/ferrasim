//! Small-strain isotropic elasticity and constant-strain TET4 element operators.
//!
//! Element-level numerical verification does not qualify a global solver.

mod material;
mod tet4;

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
