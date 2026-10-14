//! TET4 meshes, physical regions, geometric integrity and model binding.

mod binding;
mod geometry;
mod types;

pub use binding::{BindingCode, BindingErrors, BindingIssue, ModelMeshError, NodeGroupError};
pub use geometry::{MeshCode, MeshErrors, MeshIssue, RELATIVE_DEGENERACY_TOLERANCE};
pub use types::*;
