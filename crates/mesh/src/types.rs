use std::collections::BTreeMap;

use ferrasim_model::{GroupId, NodeId};

/// Positive external element ID, unique across triangles and tetrahedra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElementId(
    /// External file identifier, not a contiguous index.
    pub u64,
);

/// Named physical group as declared by Gmsh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalGroup {
    /// Topological dimension, 2 or 3 in this importer.
    pub dimension: u8,
    /// Positive physical tag, scoped by dimension.
    pub tag: u64,
    /// Named region.
    pub name: GroupId,
}

/// Three-node physical surface triangle, with original file winding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triangle {
    /// External element identifier.
    pub id: ElementId,
    /// Original node order.
    pub nodes: [NodeId; 3],
    /// Optional nonzero physical group tag.
    pub physical_tag: Option<u64>,
    /// Optional nonzero elementary entity tag.
    pub entity_tag: Option<u64>,
}

/// Four-node tetrahedron, whose positive orientation is required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tet4 {
    /// External element identifier.
    pub id: ElementId,
    /// Original node order; no implicit reorientation.
    pub nodes: [NodeId; 4],
    /// Optional nonzero physical volume tag.
    pub physical_tag: Option<u64>,
    /// Optional nonzero elementary volume tag.
    pub entity_tag: Option<u64>,
}

/// Unvalidated mesh records, also constructible directly from Rust.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// External node IDs mapped to Cartesian coordinates.
    pub nodes: BTreeMap<NodeId, [f64; 3]>,
    /// Surface triangle records.
    pub triangles: BTreeMap<ElementId, Triangle>,
    /// Volume tetrahedron records.
    pub tetrahedra: BTreeMap<ElementId, Tet4>,
    /// Physical names; checked for duplicate tags/names during validation.
    pub physical_groups: Vec<PhysicalGroup>,
}

/// Checked geometry of one tetrahedron.
#[derive(Debug, Clone, PartialEq)]
pub struct TetGeometry {
    /// Positive physical volume, in the mesh's length units cubed.
    pub volume: f64,
    /// Jacobian determinant divided by longest edge cubed.
    pub normalized_determinant: f64,
}

/// Boundary triangle metadata derived from its owning tetrahedron.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryFace {
    /// Surface triangle identifier.
    pub triangle: ElementId,
    /// Unique adjacent tetrahedron.
    pub owner: ElementId,
    /// Outward-oriented node order, independent of source winding.
    pub outward_nodes: [NodeId; 3],
    /// Outward unit normal.
    pub normal: [f64; 3],
    /// Positive triangle area.
    pub area: f64,
    /// Whether the original triangle winding was outward.
    pub source_is_outward: bool,
}

/// Mesh with immutable verified geometry and named memberships.
#[derive(Debug, Clone)]
pub struct ValidatedMesh {
    pub(crate) mesh: Mesh,
    pub(crate) geometry: BTreeMap<ElementId, TetGeometry>,
    pub(crate) boundary_faces: BTreeMap<ElementId, BoundaryFace>,
    pub(crate) surfaces: BTreeMap<GroupId, Vec<ElementId>>,
    pub(crate) volumes: BTreeMap<GroupId, Vec<ElementId>>,
}

impl ValidatedMesh {
    /// Source records, without mutable access invalidating the checks.
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// Geometry indexed by external tetrahedron ID.
    pub fn geometry(&self) -> &BTreeMap<ElementId, TetGeometry> {
        &self.geometry
    }

    /// Explicitly recorded external triangles and their outward geometry.
    pub fn boundary_faces(&self) -> &BTreeMap<ElementId, BoundaryFace> {
        &self.boundary_faces
    }

    /// Physical surface names and member triangle IDs, including empty groups.
    pub fn surface_groups(&self) -> &BTreeMap<GroupId, Vec<ElementId>> {
        &self.surfaces
    }

    /// Physical volume names and member tetrahedron IDs, including empty groups.
    pub fn volume_groups(&self) -> &BTreeMap<GroupId, Vec<ElementId>> {
        &self.volumes
    }
}
