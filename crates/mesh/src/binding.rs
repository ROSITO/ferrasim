use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ferrasim_model::{GroupId, Load, NodeId, Simulation, ValidationErrors};
use thiserror::Error;

use crate::{ElementId, ValidatedMesh};

/// Failures resolving a derived named node group.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NodeGroupError {
    /// No physical surface or volume has this name.
    #[error("unknown node group {0:?}")]
    Missing(GroupId),
    /// Both a surface and a volume share this name; source cannot be inferred.
    #[error("ambiguous node group {0:?}: both surface and volume exist")]
    Ambiguous(GroupId),
}

/// Categories of model-to-mesh mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingCode {
    /// Expected physical group is absent.
    UnknownGroup,
    /// Node group source is ambiguous between dimensions.
    AmbiguousGroup,
    /// A used selection has no members.
    EmptyGroup,
    /// Nodal load references an absent node.
    UnknownNode,
    /// A tetrahedron is not covered by model material assignments.
    UnassignedElement,
    /// Traction selection includes internal triangles.
    InteriorSurface,
    /// Overlapping node groups prescribe incompatible displacements.
    ConflictingPrescription,
}

/// Model-to-mesh diagnostic with field and affected entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingIssue {
    /// Stable category.
    pub code: BindingCode,
    /// Model field path.
    pub path: String,
    /// Affected node, if applicable.
    pub node: Option<NodeId>,
    /// Affected element, if applicable.
    pub element: Option<ElementId>,
    /// Explanation.
    pub message: String,
}

/// Collected binding failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingErrors {
    /// Individual failures in deterministic order.
    pub issues: Vec<BindingIssue>,
}
impl fmt::Display for BindingErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "model/mesh binding failed ({} issues)",
            self.issues.len()
        )
    }
}
impl std::error::Error for BindingErrors {}

/// Model validation or mesh binding failure.
#[derive(Debug, Error)]
pub enum ModelMeshError {
    /// Invalid model contract.
    #[error(transparent)]
    Model(#[from] ValidationErrors),
    /// Mesh does not satisfy model references.
    #[error(transparent)]
    Binding(#[from] BindingErrors),
}

fn issue(
    errors: &mut Vec<BindingIssue>,
    code: BindingCode,
    path: impl Into<String>,
    node: Option<NodeId>,
    element: Option<ElementId>,
    message: impl Into<String>,
) {
    errors.push(BindingIssue {
        code,
        path: path.into(),
        node,
        element,
        message: message.into(),
    });
}

impl ValidatedMesh {
    /// Derive unique sorted node IDs from a physical surface or volume name.
    ///
    /// Identical names in both dimensions are rejected, never silently unioned.
    pub fn node_group_nodes(&self, name: &GroupId) -> Result<Vec<NodeId>, NodeGroupError> {
        let mut nodes = BTreeSet::new();
        match (self.surfaces.get(name), self.volumes.get(name)) {
            (Some(_), Some(_)) => return Err(NodeGroupError::Ambiguous(name.clone())),
            (Some(triangles), None) => {
                for id in triangles {
                    nodes.extend(self.mesh.triangles[id].nodes.iter().cloned());
                }
            }
            (None, Some(tets)) => {
                for id in tets {
                    nodes.extend(self.mesh.tetrahedra[id].nodes.iter().cloned());
                }
            }
            (None, None) => return Err(NodeGroupError::Missing(name.clone())),
        }
        Ok(nodes.into_iter().collect())
    }

    /// Validate model references against immutable imported topology.
    ///
    /// All imported tetrahedra require material coverage. This does not prove
    /// absence of rigid modes or numerical solvability.
    pub fn validate_model(&self, model: &Simulation) -> Result<(), ModelMeshError> {
        model.validate()?;
        let mut errors = Vec::new();
        let mut node_groups = BTreeMap::new();
        for (index, name) in model.mesh.groups.nodes.iter().enumerate() {
            let path = format!("/mesh/groups/nodes/{index}");
            match self.node_group_nodes(name) {
                Ok(nodes) => {
                    node_groups.insert(name, nodes);
                }
                Err(NodeGroupError::Missing(_)) => issue(
                    &mut errors,
                    BindingCode::UnknownGroup,
                    path,
                    None,
                    None,
                    format!("no physical group provides nodes for {:?}", name.0),
                ),
                Err(NodeGroupError::Ambiguous(_)) => issue(
                    &mut errors,
                    BindingCode::AmbiguousGroup,
                    path,
                    None,
                    None,
                    format!("surface and volume share node-group name {:?}", name.0),
                ),
            }
        }
        for (index, name) in model.mesh.groups.surfaces.iter().enumerate() {
            if !self.surfaces.contains_key(name) {
                issue(
                    &mut errors,
                    BindingCode::UnknownGroup,
                    format!("/mesh/groups/surfaces/{index}"),
                    None,
                    None,
                    format!("missing physical surface {:?}", name.0),
                );
            }
        }
        for (index, name) in model.mesh.groups.volumes.iter().enumerate() {
            if !self.volumes.contains_key(name) {
                issue(
                    &mut errors,
                    BindingCode::UnknownGroup,
                    format!("/mesh/groups/volumes/{index}"),
                    None,
                    None,
                    format!("missing physical volume {:?}", name.0),
                );
            }
        }
        let mut covered = BTreeSet::new();
        for (index, assignment) in model.material_assignments.iter().enumerate() {
            if let Some(elements) = self.volumes.get(&assignment.region) {
                if elements.is_empty() {
                    issue(
                        &mut errors,
                        BindingCode::EmptyGroup,
                        format!("/material_assignments/{index}/region"),
                        None,
                        None,
                        "material volume selection is empty",
                    );
                }
                covered.extend(elements.iter().copied());
            }
        }
        for element in self.mesh.tetrahedra.keys() {
            if !covered.contains(element) {
                issue(
                    &mut errors,
                    BindingCode::UnassignedElement,
                    "/material_assignments",
                    None,
                    Some(*element),
                    "tetrahedron is not covered by material assignments",
                );
            }
        }
        let mut prescribed = BTreeMap::new();
        for (index, boundary) in model.boundary_conditions.iter().enumerate() {
            if let Some(nodes) = node_groups.get(&boundary.node_group) {
                if nodes.is_empty() {
                    issue(
                        &mut errors,
                        BindingCode::EmptyGroup,
                        format!("/boundary_conditions/{index}/node_group"),
                        None,
                        None,
                        "prescribed node selection is empty",
                    );
                }
                let c = &boundary.components;
                for (axis, value) in [(0, c.x), (1, c.y), (2, c.z)] {
                    if let Some(value) = value {
                        for node in nodes {
                            let key = (node.clone(), axis);
                            if let Some(previous) = prescribed.get(&key) {
                                if *previous != value {
                                    issue(
                                        &mut errors,
                                        BindingCode::ConflictingPrescription,
                                        format!("/boundary_conditions/{index}/components"),
                                        Some(node.clone()),
                                        None,
                                        "overlapping groups prescribe different values on this node/component",
                                    );
                                }
                            } else {
                                prescribed.insert(key, value);
                            }
                        }
                    }
                }
            }
        }
        for (index, load) in model.loads.iter().enumerate() {
            match load {
                Load::NodalForce { node, .. } => {
                    if !self.mesh.nodes.contains_key(node) {
                        issue(
                            &mut errors,
                            BindingCode::UnknownNode,
                            format!("/loads/{index}/node"),
                            Some(node.clone()),
                            None,
                            "nodal load references absent node",
                        );
                    }
                }
                Load::SurfaceTraction { surface_group, .. } => {
                    if let Some(triangles) = self.surfaces.get(surface_group) {
                        if triangles.is_empty() {
                            issue(
                                &mut errors,
                                BindingCode::EmptyGroup,
                                format!("/loads/{index}/surface_group"),
                                None,
                                None,
                                "traction selection is empty",
                            );
                        }
                        for triangle in triangles {
                            if !self.boundary_faces.contains_key(triangle) {
                                issue(
                                    &mut errors,
                                    BindingCode::InteriorSurface,
                                    format!("/loads/{index}/surface_group"),
                                    None,
                                    Some(*triangle),
                                    "traction triangle is an internal interface, not an exterior boundary",
                                );
                            }
                        }
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(BindingErrors { issues: errors }.into())
        }
    }
}
