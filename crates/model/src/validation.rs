use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{GroupId, Load, Material, SCHEMA_VERSION, Simulation, Units};

/// Stable categories of model-level validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    /// Unsupported schema revision.
    UnsupportedVersion,
    /// Empty identifier or collection, or an ambiguous label.
    EmptyValue,
    /// Duplicate identifier, assignment or output format.
    Duplicate,
    /// Reference absent from the declared catalog.
    UnknownReference,
    /// Declared volume without material.
    MissingAssignment,
    /// NaN or infinity in an API-created model.
    NonFinite,
    /// Material, node ID or numerical parameter outside its accepted range.
    OutOfRange,
    /// Empty prescription or conflicting component on the same group.
    InvalidPrescription,
    /// Nonportable or empty mesh source path.
    InvalidPath,
}

/// One actionable model issue, identified by JSON-pointer-like field path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Stable machine-readable issue code.
    pub code: IssueCode,
    /// Location in the model, for example `/materials/0/young`.
    pub path: String,
    /// Human-readable detail including affected identifiers where possible.
    pub message: String,
}

/// All model-level validation issues found in a deterministic traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    /// Individual issues; never empty when returned by validation.
    pub issues: Vec<ValidationIssue>,
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "model validation failed ({} issues)",
            self.issues.len()
        )
    }
}

impl std::error::Error for ValidationErrors {}

struct Checks {
    issues: Vec<ValidationIssue>,
}

impl Checks {
    fn issue(&mut self, code: IssueCode, path: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue {
            code,
            path: path.into(),
            message: message.into(),
        });
    }

    fn label(&mut self, value: &str, path: &str) {
        if value.trim().is_empty() || value.trim() != value || value.chars().any(char::is_control) {
            self.issue(
                IssueCode::EmptyValue,
                path,
                "label must be nonempty, trimmed and control-free",
            );
        }
    }

    fn finite(&mut self, value: f64, path: &str) -> bool {
        if value.is_finite() {
            true
        } else {
            self.issue(IssueCode::NonFinite, path, "value must be finite");
            false
        }
    }

    fn groups<'a>(&mut self, groups: &'a [GroupId], path: &str) -> BTreeSet<&'a GroupId> {
        let mut seen = BTreeSet::new();
        for (index, group) in groups.iter().enumerate() {
            let field = format!("{path}/{index}");
            self.label(&group.0, &field);
            if !seen.insert(group) {
                self.issue(
                    IssueCode::Duplicate,
                    field,
                    format!("duplicate group {:?}", group.0),
                );
            }
        }
        seen
    }
}

impl Simulation {
    /// Check input ranges and declared references without reading a mesh.
    ///
    /// Mesh existence, node membership, group emptiness, overlapping selections
    /// and mechanical solvability must be checked by later stages.
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut checks = Checks { issues: Vec::new() };
        if self.schema_version != SCHEMA_VERSION {
            checks.issue(
                IssueCode::UnsupportedVersion,
                "/schema_version",
                "supported version is 0.1",
            );
        }
        if let Units::Coherent(units) = &self.units {
            checks.label(&units.length, "/units/length");
            checks.label(&units.force, "/units/force");
        }
        let path = &self.mesh.path;
        if path.trim().is_empty()
            || path.trim() != path
            || path.starts_with('/')
            || path.contains(['\\', ':'])
            || path.chars().any(char::is_control)
            || path.ends_with('/')
            || matches!(path.rsplit('/').next(), Some("." | ".."))
        {
            checks.issue(
                IssueCode::InvalidPath,
                "/mesh/path",
                "use a nonempty relative file path with forward slashes",
            );
        }
        let nodes = checks.groups(&self.mesh.groups.nodes, "/mesh/groups/nodes");
        let surfaces = checks.groups(&self.mesh.groups.surfaces, "/mesh/groups/surfaces");
        let volumes = checks.groups(&self.mesh.groups.volumes, "/mesh/groups/volumes");
        if volumes.is_empty() {
            checks.issue(
                IssueCode::EmptyValue,
                "/mesh/groups/volumes",
                "at least one volume group is required",
            );
        }
        let mut materials = BTreeSet::new();
        if self.materials.is_empty() {
            checks.issue(
                IssueCode::EmptyValue,
                "/materials",
                "at least one material is required",
            );
        }
        for (index, material) in self.materials.iter().enumerate() {
            let base = format!("/materials/{index}");
            checks.label(&material.id().0, &format!("{base}/id"));
            if !materials.insert(material.id()) {
                checks.issue(
                    IssueCode::Duplicate,
                    format!("{base}/id"),
                    "duplicate material identifier",
                );
            }
            let Material::IsotropicLinearElastic { young, poisson, .. } = material;
            if checks.finite(*young, &format!("{base}/young")) && *young <= 0.0 {
                checks.issue(
                    IssueCode::OutOfRange,
                    format!("{base}/young"),
                    "Young's modulus must be positive",
                );
            }
            if checks.finite(*poisson, &format!("{base}/poisson"))
                && !(-1.0 < *poisson && *poisson < 0.5)
            {
                checks.issue(
                    IssueCode::OutOfRange,
                    format!("{base}/poisson"),
                    "Poisson's ratio must satisfy -1 < nu < 0.5",
                );
            }
        }
        let mut assigned = BTreeSet::new();
        for (index, assignment) in self.material_assignments.iter().enumerate() {
            let base = format!("/material_assignments/{index}");
            if !volumes.contains(&assignment.region) {
                checks.issue(
                    IssueCode::UnknownReference,
                    format!("{base}/region"),
                    format!("unknown volume group {:?}", assignment.region.0),
                );
            }
            if !materials.contains(&assignment.material) {
                checks.issue(
                    IssueCode::UnknownReference,
                    format!("{base}/material"),
                    format!("unknown material {:?}", assignment.material.0),
                );
            }
            if !assigned.insert(&assignment.region) {
                checks.issue(
                    IssueCode::Duplicate,
                    format!("{base}/region"),
                    "volume group assigned more than once",
                );
            }
        }
        for volume in &volumes {
            if !assigned.contains(volume) {
                checks.issue(
                    IssueCode::MissingAssignment,
                    "/material_assignments",
                    format!("volume group {:?} has no material", volume.0),
                );
            }
        }
        let mut prescriptions = BTreeMap::new();
        for (index, boundary) in self.boundary_conditions.iter().enumerate() {
            let base = format!("/boundary_conditions/{index}");
            if !nodes.contains(&boundary.node_group) {
                checks.issue(
                    IssueCode::UnknownReference,
                    format!("{base}/node_group"),
                    format!("unknown node group {:?}", boundary.node_group.0),
                );
            }
            let components = &boundary.components;
            if components.x.is_none() && components.y.is_none() && components.z.is_none() {
                checks.issue(
                    IssueCode::InvalidPrescription,
                    format!("{base}/components"),
                    "at least one prescribed component is required",
                );
            }
            for (axis, value) in [
                ("x", components.x),
                ("y", components.y),
                ("z", components.z),
            ] {
                if let Some(value) = value {
                    let field = format!("{base}/components/{axis}");
                    if checks.finite(value, &field) {
                        let key = (&boundary.node_group, axis);
                        if let Some(previous) = prescriptions.get(&key) {
                            if *previous != value {
                                checks.issue(
                                    IssueCode::InvalidPrescription,
                                    field,
                                    "conflicting values on the same group and component",
                                );
                            }
                        } else {
                            prescriptions.insert(key, value);
                        }
                    }
                }
            }
        }
        for (index, load) in self.loads.iter().enumerate() {
            let base = format!("/loads/{index}");
            let components = match load {
                Load::NodalForce { node, components } => {
                    if node.0 == 0 {
                        checks.issue(
                            IssueCode::OutOfRange,
                            format!("{base}/node"),
                            "node identifiers must be positive",
                        );
                    }
                    components
                }
                Load::SurfaceTraction {
                    surface_group,
                    components,
                } => {
                    if !surfaces.contains(surface_group) {
                        checks.issue(
                            IssueCode::UnknownReference,
                            format!("{base}/surface_group"),
                            format!("unknown surface group {:?}", surface_group.0),
                        );
                    }
                    components
                }
            };
            for (axis, value) in [
                ("x", components.x),
                ("y", components.y),
                ("z", components.z),
            ] {
                checks.finite(value, &format!("{base}/components/{axis}"));
            }
        }
        let settings = &self.solver_settings;
        for (name, value) in [("rtol", settings.rtol), ("atol", settings.atol)] {
            let field = format!("/solver_settings/{name}");
            if checks.finite(value, &field) && value < 0.0 {
                checks.issue(
                    IssueCode::OutOfRange,
                    field,
                    "tolerance must be nonnegative",
                );
            }
        }
        if settings.rtol == 0.0 && settings.atol == 0.0 {
            checks.issue(
                IssueCode::OutOfRange,
                "/solver_settings",
                "at least one tolerance must be positive",
            );
        }
        if settings.max_iterations == 0 {
            checks.issue(
                IssueCode::OutOfRange,
                "/solver_settings/max_iterations",
                "iteration budget must be positive",
            );
        }
        if self.outputs.formats.is_empty() {
            checks.issue(
                IssueCode::EmptyValue,
                "/outputs/formats",
                "at least one output is required",
            );
        }
        let mut formats = BTreeSet::new();
        for (index, format) in self.outputs.formats.iter().enumerate() {
            if !formats.insert(format) {
                checks.issue(
                    IssueCode::Duplicate,
                    format!("/outputs/formats/{index}"),
                    "duplicate output format",
                );
            }
        }
        if checks.issues.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors {
                issues: checks.issues,
            })
        }
    }
}
