use serde::{Deserialize, Serialize};

macro_rules! identifier {
    ($name:ident, $description:literal, $inner:ty) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(
            #[doc = "External identifier value, checked during validation."] pub $inner,
        );
    };
}

identifier!(
    MaterialId,
    "Stable material identifier within a model revision.",
    String
);
identifier!(
    GroupId,
    "Named group identifier within its entity category.",
    String
);
identifier!(
    NodeId,
    "Positive external mesh node identifier; not an array index.",
    u64
);

/// Named coherent unit system supported by this schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamedUnits {
    /// Metres, newtons and pascals; strains are dimensionless.
    #[serde(rename = "SI")]
    Si,
}

/// Labels for a user-declared coherent unit system, without conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoherentUnits {
    /// Length unit label, for example `mm`.
    pub length: String,
    /// Force unit label, for example `N`; stress uses force/length².
    pub force: String,
}

/// Unit declaration; custom labels do not imply automatic conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Units {
    /// Predefined unit system.
    Named(NamedUnits),
    /// User-declared consistent units.
    Coherent(CoherentUnits),
}

/// Named selections expected from the imported mesh.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Groups {
    /// Node groups, including groups derived from physical surfaces.
    pub nodes: Vec<GroupId>,
    /// Physical surface groups.
    pub surfaces: Vec<GroupId>,
    /// Physical volume groups included in the analysis.
    pub volumes: Vec<GroupId>,
}

/// Mesh file and selection declarations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeshReference {
    /// Portable slash-separated path relative to the model file.
    pub path: String,
    /// Declared selections; actual membership is verified after mesh import.
    pub groups: Groups,
}

/// Isotropic linear elastic material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Material {
    /// Elastic response defined by Young's modulus and Poisson's ratio.
    #[serde(rename = "isotropic_linear_elastic")]
    IsotropicLinearElastic {
        /// Stable material identifier.
        id: MaterialId,
        /// Young's modulus in the declared force/length² units.
        young: f64,
        /// Poisson's ratio, strictly between −1 and 0.5.
        poisson: f64,
    },
}

impl Material {
    /// Return the material identifier independently of its constitutive type.
    pub fn id(&self) -> &MaterialId {
        match self {
            Self::IsotropicLinearElastic { id, .. } => id,
        }
    }
}

/// Assign one material to a declared volumetric region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialAssignment {
    /// Declared volume group.
    pub region: GroupId,
    /// Existing material identifier.
    pub material: MaterialId,
}

/// Optional prescribed displacements; omission means the component is free.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrescribedComponents {
    /// Prescribed x displacement; `null` or absence means free.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Prescribed y displacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Prescribed z displacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

/// Cartesian vector with all components explicit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vector3 {
    /// x component.
    pub x: f64,
    /// y component.
    pub y: f64,
    /// z component.
    pub z: f64,
}

/// Kinematic prescription on a named node group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryCondition {
    /// Declared node group.
    pub node_group: GroupId,
    /// Components to prescribe.
    pub components: PrescribedComponents,
}

/// External mechanical load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Load {
    /// Force applied to one external mesh node ID.
    #[serde(rename = "nodal_force")]
    NodalForce {
        /// Positive mesh node identifier.
        node: NodeId,
        /// Force vector, not a total redistributed over a group.
        components: Vector3,
    },
    /// Uniform traction integrated on a named boundary surface.
    #[serde(rename = "surface_traction")]
    SurfaceTraction {
        /// Declared physical surface group.
        surface_group: GroupId,
        /// Traction vector in force/length² units.
        components: Vector3,
    },
}

/// Analysis supported by the MVP schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Analysis {
    /// Three-dimensional, small-strain, linear static elasticity.
    #[serde(rename = "linear_static_3d")]
    LinearStatic3d {},
}

/// Linear solver selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolverMethod {
    /// Conjugate gradient for the constrained SPD system.
    Cg,
}

/// Preconditioner selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preconditioner {
    /// Diagonal preconditioning.
    Jacobi,
}

/// Explicit numerical stopping parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolverSettings {
    /// Solver method.
    pub method: SolverMethod,
    /// Preconditioner.
    pub preconditioner: Preconditioner,
    /// Nonnegative relative residual tolerance.
    pub rtol: f64,
    /// Nonnegative absolute residual tolerance; both cannot be zero.
    pub atol: f64,
    /// Positive iteration budget.
    pub max_iterations: u64,
}

/// Output format requested for the future result exporters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    /// VTK unstructured-grid XML.
    Vtu,
    /// Comma-separated field tables.
    Csv,
    /// JSON report.
    Json,
}

/// Explicit result output requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outputs {
    /// Nonempty, nonduplicated list of supported formats.
    pub formats: Vec<OutputFormat>,
}

/// Versioned simulation description, independent of a solver and user interface.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Simulation {
    /// Exact schema version, currently `0.1`.
    pub schema_version: String,
    /// Coherent unit declaration; no automatic conversion.
    pub units: Units,
    /// Mesh source and expected groups.
    pub mesh: MeshReference,
    /// Material definitions.
    pub materials: Vec<Material>,
    /// One material assignment for every declared volume.
    pub material_assignments: Vec<MaterialAssignment>,
    /// Component-wise kinematic prescriptions.
    pub boundary_conditions: Vec<BoundaryCondition>,
    /// External loads; an empty list is allowed.
    pub loads: Vec<Load>,
    /// Physical procedure.
    pub analysis: Analysis,
    /// Numerical choices.
    pub solver_settings: SolverSettings,
    /// Result requests.
    pub outputs: Outputs,
}
