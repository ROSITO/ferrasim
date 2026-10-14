use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ferrasim_model::NodeId;

use crate::{BoundaryFace, ElementId, Mesh, TetGeometry, ValidatedMesh};

/// Reject |det(J)| / longest_edge³ <= this value, independent of mesh units.
pub const RELATIVE_DEGENERACY_TOLERANCE: f64 = 1e-12;

/// Categories of mesh integrity failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshCode {
    /// Mesh has no nodes or tetrahedra.
    Empty,
    /// Invalid, mismatched or reused external ID.
    InvalidId,
    /// NaN, infinity or an unrepresentable derived geometric quantity.
    NumericRange,
    /// Connectivity points to absent nodes.
    MissingNode,
    /// A node is repeated within an element.
    RepeatedNode,
    /// Two elements have the same node connectivity.
    DuplicateTopology,
    /// Element has zero or relatively negligible volume.
    Degenerate,
    /// Tetrahedron has negative orientation.
    Inverted,
    /// Surface is not a tetrahedral face.
    OrphanSurface,
    /// Face has more than two owners, or adjacent elements are on the same side.
    InvalidAdjacency,
    /// Physical group metadata is missing or ambiguous.
    PhysicalGroup,
    /// Node does not belong to any volume element.
    UnusedNode,
}

/// An integrity problem localized to an element or node where possible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshIssue {
    /// Machine-readable failure category.
    pub code: MeshCode,
    /// Affected element if applicable.
    pub element: Option<ElementId>,
    /// Affected node if applicable.
    pub node: Option<NodeId>,
    /// Human-readable explanation.
    pub message: String,
}

/// Collected geometric and topological failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshErrors {
    /// Individual errors in deterministic order.
    pub issues: Vec<MeshIssue>,
}

impl fmt::Display for MeshErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mesh validation failed ({} issues)", self.issues.len())
    }
}
impl std::error::Error for MeshErrors {}

fn issue(
    errors: &mut Vec<MeshIssue>,
    code: MeshCode,
    element: Option<ElementId>,
    node: Option<NodeId>,
    message: impl Into<String>,
) {
    errors.push(MeshIssue {
        code,
        element,
        node,
        message: message.into(),
    });
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn norm(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}

fn sorted<const N: usize>(mut nodes: [NodeId; N]) -> [NodeId; N] {
    nodes.sort();
    nodes
}

fn parity(nodes: &[NodeId; 3]) -> bool {
    let inversions = usize::from(nodes[0] > nodes[1])
        + usize::from(nodes[0] > nodes[2])
        + usize::from(nodes[1] > nodes[2]);
    inversions.is_multiple_of(2)
}

fn faces(nodes: &[NodeId; 4]) -> [[NodeId; 3]; 4] {
    let [a, b, c, d] = nodes.clone();
    [
        [b.clone(), c.clone(), d.clone()],
        [a.clone(), d.clone(), c.clone()],
        [a.clone(), b.clone(), d],
        [a, c, b],
    ]
}

impl Mesh {
    /// Validate topology, dimensions, connectivity and normalized geometry.
    ///
    /// Does not detect all intersections between non-neighboring elements.
    pub fn into_validated(self) -> Result<ValidatedMesh, MeshErrors> {
        let mut errors = Vec::new();
        if self.nodes.is_empty() || self.tetrahedra.is_empty() {
            issue(
                &mut errors,
                MeshCode::Empty,
                None,
                None,
                "nodes and TET4 volumes are required",
            );
        }
        for (id, coordinates) in &self.nodes {
            if id.0 == 0 {
                issue(
                    &mut errors,
                    MeshCode::InvalidId,
                    None,
                    Some(id.clone()),
                    "node ID must be positive",
                );
            }
            if coordinates.iter().any(|value| !value.is_finite()) {
                issue(
                    &mut errors,
                    MeshCode::NumericRange,
                    None,
                    Some(id.clone()),
                    "coordinates must be finite",
                );
            }
        }
        let mut group_tags = BTreeMap::new();
        let mut group_names = BTreeSet::new();
        let mut surfaces = BTreeMap::new();
        let mut volumes = BTreeMap::new();
        for group in &self.physical_groups {
            if !matches!(group.dimension, 2 | 3)
                || group.tag == 0
                || group.name.0.trim().is_empty()
                || group.name.0.trim() != group.name.0
                || group.name.0.chars().any(char::is_control)
            {
                issue(
                    &mut errors,
                    MeshCode::PhysicalGroup,
                    None,
                    None,
                    "physical names require dimension 2/3, positive tag and nonempty trimmed name",
                );
            }
            if group_tags
                .insert((group.dimension, group.tag), group.name.clone())
                .is_some()
                || !group_names.insert((group.dimension, group.name.clone()))
            {
                issue(
                    &mut errors,
                    MeshCode::PhysicalGroup,
                    None,
                    None,
                    format!("duplicate physical name/tag {:?}", group.name.0),
                );
            }
            if group.dimension == 2 {
                surfaces.insert(group.name.clone(), Vec::new());
            }
            if group.dimension == 3 {
                volumes.insert(group.name.clone(), Vec::new());
            }
        }
        let mut geometry = BTreeMap::new();
        let mut used_nodes = BTreeSet::new();
        let mut tet_sets = BTreeSet::new();
        let mut owners: BTreeMap<[NodeId; 3], Vec<(ElementId, [NodeId; 3])>> = BTreeMap::new();
        for (key, tet) in &self.tetrahedra {
            if tet.physical_tag == Some(0) || tet.entity_tag == Some(0) {
                issue(
                    &mut errors,
                    MeshCode::PhysicalGroup,
                    Some(*key),
                    None,
                    "zero tags must be represented as None in the Rust API",
                );
            }
            if key.0 == 0 || key != &tet.id || self.triangles.contains_key(key) {
                issue(
                    &mut errors,
                    MeshCode::InvalidId,
                    Some(*key),
                    None,
                    "element IDs must be positive, consistent and globally unique",
                );
            }
            if !tet_sets.insert(sorted(tet.nodes.clone())) {
                issue(
                    &mut errors,
                    MeshCode::DuplicateTopology,
                    Some(*key),
                    None,
                    "duplicate tetrahedron connectivity",
                );
            }
            let unique: BTreeSet<_> = tet.nodes.iter().collect();
            if unique.len() != 4 {
                issue(
                    &mut errors,
                    MeshCode::RepeatedNode,
                    Some(*key),
                    None,
                    "tetrahedron needs four distinct nodes",
                );
            }
            let mut points = Vec::new();
            for node in &tet.nodes {
                used_nodes.insert(node.clone());
                if let Some(point) = self.nodes.get(node) {
                    points.push(*point);
                } else {
                    issue(
                        &mut errors,
                        MeshCode::MissingNode,
                        Some(*key),
                        Some(node.clone()),
                        "tetrahedron references missing node",
                    );
                }
            }
            if let Some(tag) = tet.physical_tag
                && let Some(name) = group_tags.get(&(3, tag))
            {
                volumes.entry(name.clone()).or_default().push(*key);
            } // Unnamed physical tags remain on the source element.
            if points.len() != 4
                || unique.len() != 4
                || points.iter().flatten().any(|v| !v.is_finite())
            {
                continue;
            }
            let mut length: f64 = 0.0;
            for i in 0..4 {
                for j in i + 1..4 {
                    length = length.max(norm(sub(points[i], points[j])));
                }
            }
            if !length.is_finite() || length == 0.0 {
                issue(
                    &mut errors,
                    if length.is_finite() {
                        MeshCode::Degenerate
                    } else {
                        MeshCode::NumericRange
                    },
                    Some(*key),
                    None,
                    "edge length is zero or unrepresentable",
                );
                continue;
            }
            let edges: [[f64; 3]; 3] =
                std::array::from_fn(|i| sub(points[i + 1], points[0]).map(|v| v / length));
            let determinant = dot(edges[0], cross(edges[1], edges[2]));
            if !determinant.is_finite() || determinant.abs() <= RELATIVE_DEGENERACY_TOLERANCE {
                issue(
                    &mut errors,
                    MeshCode::Degenerate,
                    Some(*key),
                    None,
                    "normalized determinant is below the relative threshold",
                );
                continue;
            }
            if determinant < 0.0 {
                issue(
                    &mut errors,
                    MeshCode::Inverted,
                    Some(*key),
                    None,
                    "negative TET4 orientation; source nodes are not reordered",
                );
                continue;
            }
            let volume = determinant / 6.0 * length * length * length;
            if !volume.is_finite() || volume <= 0.0 {
                issue(
                    &mut errors,
                    MeshCode::NumericRange,
                    Some(*key),
                    None,
                    "volume is not representable in f64",
                );
                continue;
            }
            geometry.insert(
                *key,
                TetGeometry {
                    volume,
                    normalized_determinant: determinant,
                },
            );
            for face in faces(&tet.nodes) {
                owners
                    .entry(sorted(face.clone()))
                    .or_default()
                    .push((*key, face));
            }
        }
        for node in self.nodes.keys() {
            if !used_nodes.contains(node) {
                issue(
                    &mut errors,
                    MeshCode::UnusedNode,
                    None,
                    Some(node.clone()),
                    "node does not belong to a tetrahedron",
                );
            }
        }
        for adjacent in owners.values() {
            if adjacent.len() > 2
                || (adjacent.len() == 2 && parity(&adjacent[0].1) == parity(&adjacent[1].1))
            {
                issue(
                    &mut errors,
                    MeshCode::InvalidAdjacency,
                    Some(adjacent[0].0),
                    None,
                    "nonmanifold or same-sided tetrahedral adjacency",
                );
            }
        }
        let mut triangle_sets = BTreeSet::new();
        let mut boundary_faces = BTreeMap::new();
        for (key, triangle) in &self.triangles {
            if triangle.physical_tag == Some(0) || triangle.entity_tag == Some(0) {
                issue(
                    &mut errors,
                    MeshCode::PhysicalGroup,
                    Some(*key),
                    None,
                    "zero tags must be represented as None in the Rust API",
                );
            }
            if key.0 == 0 || key != &triangle.id {
                issue(
                    &mut errors,
                    MeshCode::InvalidId,
                    Some(*key),
                    None,
                    "triangle ID must be positive and consistent",
                );
            }
            let face_key = sorted(triangle.nodes.clone());
            if !triangle_sets.insert(face_key.clone()) {
                issue(
                    &mut errors,
                    MeshCode::DuplicateTopology,
                    Some(*key),
                    None,
                    "duplicate triangle connectivity",
                );
            }
            if triangle.nodes.iter().collect::<BTreeSet<_>>().len() != 3 {
                issue(
                    &mut errors,
                    MeshCode::RepeatedNode,
                    Some(*key),
                    None,
                    "triangle needs three distinct nodes",
                );
            }
            for node in &triangle.nodes {
                if !self.nodes.contains_key(node) {
                    issue(
                        &mut errors,
                        MeshCode::MissingNode,
                        Some(*key),
                        Some(node.clone()),
                        "triangle references missing node",
                    );
                }
            }
            if let Some(tag) = triangle.physical_tag
                && let Some(name) = group_tags.get(&(2, tag))
            {
                surfaces.entry(name.clone()).or_default().push(*key);
            } // No synthetic name is invented for unnamed physical tags.
            match owners.get(&face_key) {
                Some(adjacent) if adjacent.len() == 1 => {
                    let (owner, face) = &adjacent[0];
                    let [a, b, c] = face.clone().map(|node| self.nodes[&node]);
                    let edge1 = sub(b, a);
                    let edge2 = sub(c, a);
                    let scale = norm(edge1).max(norm(edge2));
                    let normal = cross(edge1.map(|v| v / scale), edge2.map(|v| v / scale));
                    let normal_length = norm(normal);
                    let area = normal_length / 2.0 * scale * scale;
                    if !area.is_finite()
                        || area <= 0.0
                        || !normal_length.is_finite()
                        || normal_length == 0.0
                    {
                        issue(
                            &mut errors,
                            MeshCode::NumericRange,
                            Some(*key),
                            None,
                            "boundary triangle area/normal is unrepresentable",
                        );
                    } else {
                        boundary_faces.insert(
                            *key,
                            BoundaryFace {
                                triangle: *key,
                                owner: *owner,
                                outward_nodes: face.clone(),
                                normal: normal.map(|v| v / normal_length),
                                area,
                                source_is_outward: parity(face) == parity(&triangle.nodes),
                            },
                        );
                    }
                }
                Some(adjacent) if adjacent.len() == 2 => {} // Valid internal interface, not a boundary load surface.
                _ => issue(
                    &mut errors,
                    MeshCode::OrphanSurface,
                    Some(*key),
                    None,
                    "triangle is not a face of valid TET4 elements",
                ),
            }
        }
        if errors.is_empty() {
            Ok(ValidatedMesh {
                mesh: self,
                geometry,
                boundary_faces,
                surfaces,
                volumes,
            })
        } else {
            Err(MeshErrors { issues: errors })
        }
    }
}
