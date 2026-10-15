use crate::{AssembledSystem, DofMap, FemError};
use ferrasim_mesh::ValidatedMesh;
use ferrasim_model::{Load, Simulation, SolverSettings};
use ferrasim_solvers::SolveReport;

fn finite(value: f64) -> Result<f64, FemError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FemError::NumericRange(
            "load or balance arithmetic overflow",
        ))
    }
}
fn add(target: &mut f64, value: f64) -> Result<(), FemError> {
    *target = finite(*target + value)?;
    Ok(())
}
pub(crate) fn integrate_loads(
    mesh: &ValidatedMesh,
    model: &Simulation,
    dofs: &DofMap,
) -> Result<Vec<f64>, FemError> {
    let mut forces = vec![0.0; dofs.len()];
    for load in &model.loads {
        match load {
            Load::NodalForce { node, components } => {
                for (axis, value) in [components.x, components.y, components.z]
                    .into_iter()
                    .enumerate()
                {
                    add(&mut forces[dofs.index(node, axis)?], value)?;
                }
            }
            Load::SurfaceTraction {
                surface_group,
                components,
            } => {
                let triangles = mesh
                    .surface_groups()
                    .get(surface_group)
                    .ok_or(FemError::Assembly("unknown traction surface"))?;
                if triangles.is_empty() {
                    return Err(FemError::Assembly("empty traction surface"));
                }
                for id in triangles {
                    let face = mesh
                        .boundary_faces()
                        .get(id)
                        .ok_or(FemError::Assembly("traction requires an exterior face"))?;
                    let weight = face.area / 3.0;
                    if weight == 0.0 {
                        return Err(FemError::NumericRange(
                            "traction integration weight underflow",
                        ));
                    }
                    for (axis, traction) in [components.x, components.y, components.z]
                        .into_iter()
                        .enumerate()
                    {
                        let nodal = finite(traction * weight)?;
                        if traction != 0.0 && nodal == 0.0 {
                            return Err(FemError::NumericRange("traction contribution underflow"));
                        }
                        for node in &face.outward_nodes {
                            add(&mut forces[dofs.index(node, axis)?], nodal)?;
                        }
                    }
                }
            }
        }
    }
    Ok(forces)
}

/// Positive, caller-declared force and length scales, in the model's coherent units.
#[derive(Debug, Clone, Copy)]
pub struct BalanceScales {
    force: f64,
    length: f64,
    moment: f64,
}
impl BalanceScales {
    /// Construct finite nonzero force, length and force-times-length scales.
    pub fn new(force: f64, length: f64) -> Result<Self, FemError> {
        let moment = force * length;
        if !(force.is_finite() && length.is_finite() && moment.is_finite())
            || force <= 0.0
            || length <= 0.0
            || moment <= 0.0
        {
            return Err(FemError::Assembly(
                "balance force and length scales must be positive and representable",
            ));
        }
        Ok(Self {
            force,
            length,
            moment,
        })
    }
    /// Declared force scale.
    pub fn force(&self) -> f64 {
        self.force
    }
    /// Declared length scale.
    pub fn length(&self) -> f64 {
        self.length
    }
}

/// Global balance about an explicitly recorded origin; no acceptance threshold is hidden.
#[derive(Debug, Clone)]
pub struct MechanicalBalance {
    /// Moment origin in global coordinates and coherent length units.
    pub origin: [f64; 3],
    /// Scales used to normalize this balance.
    pub scales: BalanceScales,
    /// Sum of applied nodal and consistently integrated surface forces.
    pub external_force: [f64; 3],
    /// Sum of reactions on prescribed components only.
    pub reaction_force: [f64; 3],
    /// Applied plus reaction force.
    pub force_imbalance: [f64; 3],
    /// Moment of applied forces about origin.
    pub external_moment: [f64; 3],
    /// Moment of constrained reactions about origin.
    pub reaction_moment: [f64; 3],
    /// Applied plus reaction moment.
    pub moment_imbalance: [f64; 3],
    /// norm(force_imbalance) / force_scale.
    pub relative_force_error: f64,
    /// norm(moment_imbalance) / (force_scale * length_scale).
    pub relative_moment_error: f64,
    /// Norm of unconstrained residual components divided by force_scale.
    pub relative_free_residual: f64,
    /// Norm of prescribed displacement mismatches divided by length_scale.
    pub relative_prescription_error: f64,
    /// Elastic energy 0.5 u^T K_original u.
    pub strain_energy: f64,
    /// Final applied-force work u^T F (twice the ramped-load work).
    pub load_work: f64,
    /// Final reaction work on prescribed components u_c^T R_c.
    pub prescription_work: f64,
    /// Normalized abs(2*energy - load_work - prescription_work).
    pub relative_energy_error: f64,
}

/// Successful model-load solve and its full-field mechanical balance.
#[derive(Debug, Clone)]
pub struct StaticSolution {
    /// CG/Jacobi report for the reduced system.
    pub solver: SolveReport,
    /// Full displacements in node-major xyz order.
    pub displacements: Vec<f64>,
    /// Reactions with constrained global DOF indices, in sorted order.
    pub reactions: Vec<(usize, f64)>,
    /// Force, moment, free residual and energy diagnostics.
    pub balance: MechanicalBalance,
}

fn dot(a: &[f64], b: &[f64]) -> Result<f64, FemError> {
    let mut result = 0.0;
    for (a, b) in a.iter().zip(b) {
        add(&mut result, finite(a * b)?)?;
    }
    Ok(result)
}
fn norm(v: &[f64]) -> Result<f64, FemError> {
    finite(v.iter().fold(0.0_f64, |n, x| n.hypot(*x)))
}
fn resultant(
    coordinates: &[[f64; 3]],
    forces: &[f64],
    origin: [f64; 3],
) -> Result<([f64; 3], [f64; 3]), FemError> {
    let mut force = [0.0; 3];
    let mut moment = [0.0; 3];
    for (point, f) in coordinates.iter().zip(forces.chunks_exact(3)) {
        let mut arm = [0.0; 3];
        for i in 0..3 {
            arm[i] = finite(point[i] - origin[i])?;
            add(&mut force[i], f[i])?;
        }
        for i in 0..3 {
            let value = finite(
                finite(arm[(i + 1) % 3] * f[(i + 2) % 3])?
                    - finite(arm[(i + 2) % 3] * f[(i + 1) % 3])?,
            )?;
            add(&mut moment[i], value)?;
        }
    }
    Ok((force, moment))
}

impl AssembledSystem {
    /// Eliminate the prescriptions using forces integrated from this system's model.
    pub fn reduce_model_loads(&self) -> Result<crate::ReducedSystem, FemError> {
        self.reduce(self.forces())
    }

    /// Solve the actual model loads, then compute reactions and full mechanical diagnostics.
    /// A CG success certifies its residual; balance metrics remain explicit for assessment.
    pub fn solve_model_loads(
        &self,
        settings: &SolverSettings,
        origin: [f64; 3],
        scales: BalanceScales,
    ) -> Result<StaticSolution, FemError> {
        let reduced = self.reduce_model_loads()?;
        let solver = reduced.solve(settings)?;
        let displacements = reduced.reconstruct(&solver.solution)?;
        let reactions = reduced.reactions(&solver.solution)?;
        let balance = self.balance(&displacements, origin, scales)?;
        Ok(StaticSolution {
            solver,
            displacements,
            reactions,
            balance,
        })
    }

    /// Assess a full displacement field against this system's original K and model forces.
    /// This method accepts arbitrary finite fields and does not declare convergence.
    pub fn balance(
        &self,
        displacements: &[f64],
        origin: [f64; 3],
        scales: BalanceScales,
    ) -> Result<MechanicalBalance, FemError> {
        if origin.iter().any(|v| !v.is_finite()) {
            return Err(FemError::Assembly("balance origin must be finite"));
        }
        let ku = self.stiffness().multiply(displacements)?;
        let residual: Vec<_> = ku
            .iter()
            .zip(self.forces())
            .map(|(a, b)| finite(a - b))
            .collect::<Result<_, _>>()?;
        let prescribed_error: Vec<_> = self
            .prescriptions()
            .iter()
            .map(|p| finite(displacements[p.dof] - p.value))
            .collect::<Result<_, _>>()?;
        let mut reactions = vec![0.0; self.dofs().len()];
        let mut free_residual = residual.clone();
        for p in self.prescriptions() {
            reactions[p.dof] = residual[p.dof];
            free_residual[p.dof] = 0.0;
        }
        let (external_force, external_moment) =
            resultant(self.coordinates(), self.forces(), origin)?;
        let (reaction_force, reaction_moment) = resultant(self.coordinates(), &reactions, origin)?;
        let mut force_imbalance = [0.0; 3];
        let mut moment_imbalance = [0.0; 3];
        for i in 0..3 {
            force_imbalance[i] = finite(external_force[i] + reaction_force[i])?;
            moment_imbalance[i] = finite(external_moment[i] + reaction_moment[i])?;
        }
        let twice_energy = dot(displacements, &ku)?;
        let strain_energy = finite(0.5 * twice_energy)?;
        let load_work = dot(displacements, self.forces())?;
        let prescription_work = dot(displacements, &reactions)?;
        let energy_denominator = scales
            .moment
            .max(twice_energy.abs())
            .max(finite(load_work.abs() + prescription_work.abs())?);
        let energy_gap = finite(finite(twice_energy - load_work)? - prescription_work)?.abs();
        Ok(MechanicalBalance {
            origin,
            scales,
            external_force,
            reaction_force,
            force_imbalance,
            external_moment,
            reaction_moment,
            moment_imbalance,
            relative_force_error: finite(norm(&force_imbalance)? / scales.force)?,
            relative_moment_error: finite(norm(&moment_imbalance)? / scales.moment)?,
            relative_free_residual: finite(norm(&free_residual)? / scales.force)?,
            relative_prescription_error: finite(norm(&prescribed_error)? / scales.length)?,
            strain_energy,
            load_work,
            prescription_work,
            relative_energy_error: finite(energy_gap / energy_denominator)?,
        })
    }
}
