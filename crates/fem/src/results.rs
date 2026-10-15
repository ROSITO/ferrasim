use crate::{
    AssembledSystem, BalanceScales, ElementDisplacements, FemError, IsotropicElastic,
    MaterialWarning, MechanicalBalance, Tet4Element,
};
use ferrasim_mesh::{ElementId, ValidatedMesh};
use ferrasim_model::{MaterialId, NodeId, Simulation};
use ferrasim_solvers::SolveReport;
use std::collections::BTreeMap;

/// A nodal field, in the global Cartesian frame and the model's coherent units.
#[derive(Debug, Clone)]
pub struct NodalResult {
    /// Original external node identifier, never a row offset.
    pub id: NodeId,
    /// Undeformed global coordinates.
    pub coordinates: [f64; 3],
    /// Global xyz displacements.
    pub displacement: [f64; 3],
    /// Integrated xyz external forces.
    pub applied_force: [f64; 3],
    /// Reactions; None means a free component, not a zero reaction.
    pub reaction: [Option<f64>; 3],
    /// Prescribed displacements; None means a free component.
    pub prescribed: [Option<f64>; 3],
}
/// Constant, unsmoothed field on one TET4.
#[derive(Debug, Clone)]
pub struct ElementResult {
    /// Original external tetrahedron identifier.
    pub id: ElementId,
    /// External vertex IDs in original local order.
    pub nodes: [NodeId; 4],
    /// Assigned material identifier.
    pub material: MaterialId,
    /// Positive element volume.
    pub volume: f64,
    /// Engineering strains xx, yy, zz, xy, yz, xz; shear entries are gamma.
    pub strain: [f64; 6],
    /// Cauchy stresses xx, yy, zz, xy, yz, xz.
    pub stress: [f64; 6],
    /// Nonnegative von Mises equivalent stress.
    pub von_mises: f64,
    /// Integrated elastic strain energy.
    pub energy: f64,
}
/// Immutable results tied to the exact validated model/mesh used in the solve.
/// Construction is available only through `solve_static_results`, after CG success.
#[derive(Debug, Clone)]
pub struct SimulationResults {
    model: Simulation,
    mesh: ValidatedMesh,
    nodes: Vec<NodalResult>,
    elements: Vec<ElementResult>,
    solver: SolveReport,
    balance: MechanicalBalance,
    warnings: Vec<(MaterialId, MaterialWarning)>,
}
impl SimulationResults {
    /// Exact parsed model snapshot, including settings, units and loads.
    pub fn model(&self) -> &Simulation {
        &self.model
    }
    /// Exact validated mesh snapshot, including boundary selections.
    pub fn mesh(&self) -> &ValidatedMesh {
        &self.mesh
    }
    /// Nodal fields sorted by external ID.
    pub fn nodes(&self) -> &[NodalResult] {
        &self.nodes
    }
    /// Element fields sorted by external ID.
    pub fn elements(&self) -> &[ElementResult] {
        &self.elements
    }
    /// Verified CG report for the reduced system.
    pub fn solver(&self) -> &SolveReport {
        &self.solver
    }
    /// Full mechanical balance, with origin and scales.
    pub fn balance(&self) -> &MechanicalBalance {
        &self.balance
    }
    /// Material advisories, without silent parameter changes.
    pub fn material_warnings(&self) -> &[(MaterialId, MaterialWarning)] {
        &self.warnings
    }
}
/// Von Mises for stress in xx,yy,zz,xy,yz,xz order, using scaled arithmetic.
/// Nonfinite inputs or a nonrepresentable nonzero result return an error.
pub fn von_mises(stress: &[f64; 6]) -> Result<f64, FemError> {
    if stress.iter().any(|x| !x.is_finite()) {
        return Err(FemError::NumericRange("von Mises requires finite stress"));
    }
    let scale = stress.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return Ok(0.0);
    }
    let s = stress.map(|x| x / scale);
    let normal = (s[0] - s[1]).hypot(s[1] - s[2]).hypot(s[2] - s[0]) / 2.0_f64.sqrt();
    let shear = s[3].hypot(s[4]).hypot(s[5]) * 3.0_f64.sqrt();
    let equivalent = normal.hypot(shear);
    let result = scale * equivalent;
    if !result.is_finite() || (equivalent > 0.0 && result == 0.0) {
        return Err(FemError::NumericRange(
            "von Mises result is not representable",
        ));
    }
    Ok(result)
}
/// Solve actual model loads and recover nodal/element fields from that same solve.
/// No successful results object is returned after any solve or recovery error.
pub fn solve_static_results(
    mesh: &ValidatedMesh,
    model: &Simulation,
    origin: [f64; 3],
    scales: BalanceScales,
) -> Result<SimulationResults, FemError> {
    let system = AssembledSystem::assemble(mesh, model)?;
    let solution = system.solve_model_loads(&model.solver_settings, origin, scales)?;
    let prescribed: BTreeMap<_, _> = system
        .prescriptions()
        .iter()
        .map(|p| (p.dof, p.value))
        .collect();
    let reactions: BTreeMap<_, _> = solution.reactions.iter().copied().collect();
    let nodes = system
        .dofs()
        .nodes()
        .iter()
        .enumerate()
        .map(|(i, id)| NodalResult {
            id: id.clone(),
            coordinates: system.coordinates()[i],
            displacement: std::array::from_fn(|axis| solution.displacements[3 * i + axis]),
            applied_force: std::array::from_fn(|axis| system.forces()[3 * i + axis]),
            reaction: std::array::from_fn(|axis| reactions.get(&(3 * i + axis)).copied()),
            prescribed: std::array::from_fn(|axis| prescribed.get(&(3 * i + axis)).copied()),
        })
        .collect();
    let laws: BTreeMap<_, _> = model
        .materials
        .iter()
        .map(|m| Ok((m.id(), IsotropicElastic::try_from(m)?)))
        .collect::<Result<_, FemError>>()?;
    let mut assignments = BTreeMap::new();
    for a in &model.material_assignments {
        for id in &mesh.volume_groups()[&a.region] {
            assignments.insert(*id, &a.material);
        }
    }
    let mut elements = Vec::new();
    for (id, tet) in &mesh.mesh().tetrahedra {
        let material = assignments[id];
        let mut u = ElementDisplacements::zeros();
        for i in 0..12 {
            u[i] = solution.displacements[system.dofs().index(&tet.nodes[i / 3], i % 3)?];
        }
        let element = Tet4Element::from_mesh(mesh, *id)?;
        let response = element.response(&laws[material], &u)?;
        let stress = std::array::from_fn(|i| response.stress[i]);
        elements.push(ElementResult {
            id: *id,
            nodes: tet.nodes.clone(),
            material: material.clone(),
            volume: element.volume(),
            strain: std::array::from_fn(|i| response.strain[i]),
            stress,
            von_mises: von_mises(&stress)?,
            energy: response.energy,
        });
    }
    Ok(SimulationResults {
        model: model.clone(),
        mesh: mesh.clone(),
        nodes,
        elements,
        solver: solution.solver,
        balance: solution.balance,
        warnings: system.material_warnings().to_vec(),
    })
}
