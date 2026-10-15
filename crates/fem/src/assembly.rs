use crate::{FemError, IsotropicElastic, MaterialWarning, Tet4Element};
use ferrasim_mesh::ValidatedMesh;
use ferrasim_model::{MaterialId, NodeId, Simulation};
use ferrasim_solvers::SparseMatrix;
use std::collections::BTreeMap;

/// Deterministic node-major xyz numbering, sorted by external node identifier.
#[derive(Debug, Clone)]
pub struct DofMap {
    nodes: Vec<NodeId>,
    indices: BTreeMap<NodeId, usize>,
}
impl DofMap {
    fn new(mesh: &ValidatedMesh) -> Result<Self, FemError> {
        let nodes: Vec<_> = mesh.mesh().nodes.keys().cloned().collect();
        nodes
            .len()
            .checked_mul(3)
            .ok_or(FemError::Assembly("DOF count overflow"))?;
        let indices = nodes
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, node)| (node, i * 3))
            .collect();
        Ok(Self { nodes, indices })
    }
    /// External nodes in internal order; external IDs are never array indices.
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }
    /// Total number of degrees of freedom.
    pub fn len(&self) -> usize {
        self.nodes.len() * 3
    }
    /// Whether the map has no degrees of freedom.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Index for an external node and Cartesian component (0=x, 1=y, 2=z).
    pub fn index(&self, node: &NodeId, component: usize) -> Result<usize, FemError> {
        if component >= 3 {
            return Err(FemError::Assembly("component must be 0, 1 or 2"));
        }
        self.indices
            .get(node)
            .map(|base| base + component)
            .ok_or(FemError::Assembly("unknown external node"))
    }
}

/// One exact component-wise displacement prescription.
#[derive(Debug, Clone, Copy)]
pub struct PrescribedDof {
    /// Internal global degree-of-freedom index.
    pub dof: usize,
    /// Prescribed displacement, in coherent length units.
    pub value: f64,
}

/// Global stiffness and model prescriptions; no load is interpreted at S04.
#[derive(Debug, Clone)]
pub struct AssembledSystem {
    dofs: DofMap,
    stiffness: SparseMatrix,
    prescribed: Vec<PrescribedDof>,
    warnings: Vec<(MaterialId, MaterialWarning)>,
}
impl AssembledSystem {
    /// Assemble all TET4s, requiring exactly one valid material per element.
    /// Model loads are validated but not integrated; provide the force vector to `reduce`.
    /// Nodes not used by any tetrahedron are rejected rather than adding silent null rows.
    pub fn assemble(mesh: &ValidatedMesh, model: &Simulation) -> Result<Self, FemError> {
        mesh.validate_model(model)?;
        let dofs = DofMap::new(mesh)?;
        let laws: BTreeMap<_, _> = model
            .materials
            .iter()
            .map(|material| Ok((material.id(), IsotropicElastic::try_from(material)?)))
            .collect::<Result<_, FemError>>()?;
        let warnings = laws
            .iter()
            .flat_map(|(id, law)| {
                law.warnings()
                    .into_iter()
                    .map(move |warning| ((*id).clone(), warning))
            })
            .collect();
        let mut materials = BTreeMap::new();
        for assignment in &model.material_assignments {
            let members = mesh
                .volume_groups()
                .get(&assignment.region)
                .ok_or(FemError::Assembly("missing material region"))?;
            for id in members {
                if materials.insert(*id, &laws[&assignment.material]).is_some() {
                    return Err(FemError::Assembly("ambiguous element material assignment"));
                }
            }
        }
        let mut entries = Vec::new();
        for (id, tet) in &mesh.mesh().tetrahedra {
            let law = materials
                .get(id)
                .ok_or(FemError::Assembly("unassigned element"))?;
            let local = Tet4Element::from_mesh(mesh, *id)?.stiffness(law)?;
            let mut indices = [0; 12];
            for (i, index) in indices.iter_mut().enumerate() {
                *index = dofs.index(&tet.nodes[i / 3], i % 3)?;
            }
            for i in 0..12 {
                for j in 0..12 {
                    entries.push((indices[i], indices[j], local[(i, j)]));
                }
            }
        }
        let stiffness = SparseMatrix::from_triplets(dofs.len(), entries)?;
        let mut prescribed = Vec::new();
        for condition in &model.boundary_conditions {
            let nodes = mesh
                .node_group_nodes(&condition.node_group)
                .map_err(|_| FemError::Assembly("invalid prescribed node group"))?;
            let c = &condition.components;
            for node in nodes {
                for (axis, value) in [c.x, c.y, c.z].into_iter().enumerate() {
                    if let Some(value) = value {
                        prescribed.push(PrescribedDof {
                            dof: dofs.index(&node, axis)?,
                            value,
                        });
                    }
                }
            }
        }
        // Binding validation already rejects conflicting values. Canonicalize overlaps.
        let prescribed = canonical_prescriptions(dofs.len(), &prescribed)?;
        Ok(Self {
            dofs,
            stiffness,
            prescribed,
            warnings,
        })
    }
    /// Constitutive advisories with their material identifiers, in sorted order.
    pub fn material_warnings(&self) -> &[(MaterialId, MaterialWarning)] {
        &self.warnings
    }
    /// Stable numbering.
    pub fn dofs(&self) -> &DofMap {
        &self.dofs
    }
    /// Original unmodified stiffness.
    pub fn stiffness(&self) -> &SparseMatrix {
        &self.stiffness
    }
    /// Unique prescribed DOFs, sorted by index.
    pub fn prescriptions(&self) -> &[PrescribedDof] {
        &self.prescribed
    }
    /// Eliminate model prescriptions with an explicit, fully assembled force vector.
    pub fn reduce(&self, forces: &[f64]) -> Result<ReducedSystem, FemError> {
        ReducedSystem::new(&self.stiffness, forces, &self.prescribed)
    }
}

fn canonical_prescriptions(
    size: usize,
    prescriptions: &[PrescribedDof],
) -> Result<Vec<PrescribedDof>, FemError> {
    let mut unique = BTreeMap::new();
    for p in prescriptions {
        if p.dof >= size || !p.value.is_finite() {
            return Err(FemError::Assembly("invalid prescribed DOF or value"));
        }
        if let Some(previous) = unique.insert(p.dof, p.value)
            && previous != p.value
        {
            return Err(FemError::Assembly("contradictory prescribed displacements"));
        }
    }
    Ok(unique
        .into_iter()
        .map(|(dof, value)| PrescribedDof { dof, value })
        .collect())
}

/// Elimination Kff uf = Ff - Kfc uc, retaining the original K and F for reactions.
/// No dense matrix or linear solve is constructed here.
#[derive(Debug, Clone)]
pub struct ReducedSystem {
    original: SparseMatrix,
    forces: Vec<f64>,
    stiffness: SparseMatrix,
    rhs: Vec<f64>,
    free: Vec<usize>,
    prescribed: Vec<PrescribedDof>,
}
impl ReducedSystem {
    /// Reduce a finite square matrix and vector; identical overlapping constraints coalesce.
    /// The unconstrained and fully prescribed cases are supported, including zero free DOFs.
    pub fn new(
        original: &SparseMatrix,
        forces: &[f64],
        prescribed: &[PrescribedDof],
    ) -> Result<Self, FemError> {
        let size = original.size();
        if forces.len() != size || forces.iter().any(|x| !x.is_finite()) {
            return Err(FemError::Assembly("invalid force dimension or value"));
        }
        let prescribed = canonical_prescriptions(size, prescribed)?;
        let mut values = vec![None; size];
        for p in &prescribed {
            values[p.dof] = Some(p.value);
        }
        let free: Vec<_> = (0..size).filter(|i| values[*i].is_none()).collect();
        let mut reduced_indices = vec![None; size];
        for (index, global) in free.iter().enumerate() {
            reduced_indices[*global] = Some(index);
        }
        let mut rhs: Vec<_> = free.iter().map(|global| forces[*global]).collect();
        let mut entries = Vec::new();
        for (row, col, value) in original.entries() {
            if let Some(i) = reduced_indices[row] {
                if let Some(j) = reduced_indices[col] {
                    entries.push((i, j, value));
                } else if let Some(u) = values[col] {
                    rhs[i] -= value * u;
                    if !rhs[i].is_finite() {
                        return Err(FemError::NumericRange("prescribed RHS overflow"));
                    }
                }
            }
        }
        Ok(Self {
            original: original.clone(),
            forces: forces.to_vec(),
            stiffness: SparseMatrix::from_triplets(free.len(), entries)?,
            rhs,
            free,
            prescribed,
        })
    }
    /// Reduced sparse stiffness Kff.
    pub fn stiffness(&self) -> &SparseMatrix {
        &self.stiffness
    }
    /// Reduced right-hand side, including the nonzero displacement correction.
    pub fn rhs(&self) -> &[f64] {
        &self.rhs
    }
    /// Original indices of free degrees of freedom in reduced order.
    pub fn free_dofs(&self) -> &[usize] {
        &self.free
    }
    /// Solve the reduced system using S05 CG/Jacobi and the model's explicit tolerances.
    /// A successful residual does not certify that the mechanical model is unique.
    pub fn solve(
        &self,
        settings: &ferrasim_model::SolverSettings,
    ) -> Result<ferrasim_solvers::SolveReport, ferrasim_solvers::SolveError> {
        ferrasim_solvers::solve_cg_jacobi(&self.stiffness, &self.rhs, settings)
    }
    /// Reconstruct the full displacement field in original DOF order.
    pub fn reconstruct(&self, free_solution: &[f64]) -> Result<Vec<f64>, FemError> {
        if free_solution.len() != self.free.len() || free_solution.iter().any(|x| !x.is_finite()) {
            return Err(FemError::Assembly("invalid reduced displacement vector"));
        }
        let mut full = vec![0.0; self.original.size()];
        for (i, value) in self.free.iter().zip(free_solution) {
            full[*i] = *value;
        }
        for p in &self.prescribed {
            full[p.dof] = p.value;
        }
        Ok(full)
    }
    /// Full residual K_original u - F_original: constrained entries are reactions.
    /// Free entries remain available as equilibrium diagnostics, never discarded.
    pub fn residual(&self, free_solution: &[f64]) -> Result<Vec<f64>, FemError> {
        let mut result = self.original.multiply(&self.reconstruct(free_solution)?)?;
        for (value, force) in result.iter_mut().zip(&self.forces) {
            *value -= force;
            if !value.is_finite() {
                return Err(FemError::NumericRange("reaction overflow"));
            }
        }
        Ok(result)
    }
    /// Reactions on prescribed DOFs only, in sorted global order.
    pub fn reactions(&self, free_solution: &[f64]) -> Result<Vec<(usize, f64)>, FemError> {
        let residual = self.residual(free_solution)?;
        Ok(self
            .prescribed
            .iter()
            .map(|p| (p.dof, residual[p.dof]))
            .collect())
    }
}
