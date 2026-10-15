//! Global tests against a rational dense reference generated without Rust/Voigt.
use ferrasim_fem::{
    AssembledSystem, ElementDisplacements, IsotropicElastic, PrescribedDof, ReducedSystem,
    Tet4Element,
};
use ferrasim_mesh::{ElementId, Mesh, PhysicalGroup, Tet4, Triangle, ValidatedMesh};
use ferrasim_model::{
    BoundaryCondition, GroupId, Material, MaterialAssignment, MaterialId, NodeId,
    PrescribedComponents, Simulation,
};
use ferrasim_solvers::SparseMatrix;
use nalgebra::{DMatrix, DVector};
use serde::Deserialize;

#[derive(Deserialize)]
struct Reference {
    node_ids: Vec<u64>,
    coordinates: Vec<[f64; 3]>,
    connectivities: Vec<[usize; 4]>,
    poisson: String,
    gradient: [[String; 3]; 3],
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    young: Vec<f64>,
    stiffness: Vec<Vec<String>>,
    forces: Vec<String>,
    prescribed: Vec<String>,
    reduced: Vec<Vec<String>>,
    rhs: Vec<String>,
    solution: Vec<String>,
    residual: Vec<String>,
}
fn reference() -> Reference {
    serde_json::from_str(include_str!(
        "../../../validation/fixtures/patch-reference.json"
    ))
    .unwrap()
}
fn fraction(s: &str) -> f64 {
    match s.split_once('/') {
        Some((a, b)) => a.parse::<f64>().unwrap() / b.parse::<f64>().unwrap(),
        None => s.parse().unwrap(),
    }
}
fn setup(r: &Reference, case: &Case) -> (ValidatedMesh, Simulation) {
    let mut mesh = Mesh::default();
    for (id, xyz) in r.node_ids.iter().zip(&r.coordinates) {
        mesh.nodes.insert(NodeId(*id), *xyz);
    }
    for (i, connectivity) in r.connectivities.iter().enumerate() {
        let id = ElementId(1000 + i as u64 * 11);
        mesh.tetrahedra.insert(
            id,
            Tet4 {
                id,
                nodes: connectivity.map(|j| NodeId(r.node_ids[j])),
                physical_tag: Some(i as u64 + 1),
                entity_tag: None,
            },
        );
        mesh.physical_groups.push(PhysicalGroup {
            dimension: 3,
            tag: i as u64 + 1,
            name: GroupId(format!("cell{i}")),
        });
    }
    mesh.physical_groups.push(PhysicalGroup {
        dimension: 2,
        tag: 1,
        name: GroupId("boundary".into()),
    });
    for (i, indices) in [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]]
        .into_iter()
        .enumerate()
    {
        let id = ElementId(i as u64 + 1);
        mesh.triangles.insert(
            id,
            Triangle {
                id,
                nodes: indices.map(|j| NodeId(r.node_ids[j])),
                physical_tag: Some(1),
                entity_tag: None,
            },
        );
    }
    let mut model: Simulation =
        serde_json::from_str(include_str!("../../../examples/minimal/model.json")).unwrap();
    model.materials = case
        .young
        .iter()
        .enumerate()
        .map(|(i, e)| Material::IsotropicLinearElastic {
            id: MaterialId(format!("m{i}")),
            young: *e,
            poisson: fraction(&r.poisson),
        })
        .collect();
    model.mesh.groups.volumes = (0..4).map(|i| GroupId(format!("cell{i}"))).collect();
    model.material_assignments = (0..4)
        .map(|i| MaterialAssignment {
            region: GroupId(format!("cell{i}")),
            material: MaterialId(format!("m{i}")),
        })
        .collect();
    model.mesh.groups.nodes = vec![GroupId("boundary".into())];
    model.mesh.groups.surfaces = vec![GroupId("boundary".into())];
    model.boundary_conditions.clear();
    model.loads.clear();
    (mesh.into_validated().unwrap(), model)
}
// Dense linear solving exists only in this integration-test executable.
fn dense_solve(matrix: &SparseMatrix, rhs: &[f64]) -> Vec<f64> {
    assert!(
        matrix.size() <= 96,
        "dense reference is limited to tiny tests"
    );
    if matrix.size() == 0 {
        return vec![];
    }
    let mut dense = DMatrix::zeros(matrix.size(), matrix.size());
    for (i, j, value) in matrix.entries() {
        dense[(i, j)] = value;
    }
    dense
        .lu()
        .solve(&DVector::from_column_slice(rhs))
        .unwrap()
        .as_slice()
        .to_vec()
}
fn norm(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().map(|x| x * x).sum::<f64>().sqrt()
}
fn compare(actual: &[f64], expected: &[f64], tolerance: f64) -> f64 {
    assert_eq!(actual.len(), expected.len());
    let error = norm(actual.iter().zip(expected).map(|(a, b)| a - b))
        / norm(expected.iter().copied()).max(1.0);
    assert!(error <= tolerance, "relative error {error}");
    error
}
fn internal_vector(r: &Reference, system: &AssembledSystem, values: &[String]) -> Vec<f64> {
    let mut output = vec![0.0; 15];
    for (node, id) in r.node_ids.iter().enumerate() {
        for axis in 0..3 {
            output[system.dofs().index(&NodeId(*id), axis).unwrap()] =
                fraction(&values[3 * node + axis]);
        }
    }
    output
}
fn reduction(r: &Reference, case: &Case, system: &AssembledSystem) -> ReducedSystem {
    let forces = internal_vector(r, system, &case.forces);
    let prescribed: Vec<_> = (0..4)
        .flat_map(|node| {
            (0..3).map(move |axis| PrescribedDof {
                dof: system
                    .dofs()
                    .index(&NodeId(r.node_ids[node]), axis)
                    .unwrap(),
                value: fraction(&case.prescribed[3 * node + axis]),
            })
        })
        .collect();
    ReducedSystem::new(system.stiffness(), &forces, &prescribed).unwrap()
}

#[test]
fn assembled_and_reduced_operators_match_exact_dense_reference() {
    let r = reference();
    for case in &r.cases {
        let (mesh, model) = setup(&r, case);
        let system = AssembledSystem::assemble(&mesh, &model).unwrap();
        assert_eq!(
            system.dofs().nodes(),
            &[NodeId(7), NodeId(23), NodeId(61), NodeId(90), NodeId(400)]
        );
        let mut expected = vec![0.0; 225];
        for i in 0..15 {
            for j in 0..15 {
                let row = system
                    .dofs()
                    .index(&NodeId(r.node_ids[i / 3]), i % 3)
                    .unwrap();
                let col = system
                    .dofs()
                    .index(&NodeId(r.node_ids[j / 3]), j % 3)
                    .unwrap();
                expected[15 * row + col] = fraction(&case.stiffness[i][j]);
            }
        }
        let mut actual = vec![0.0; 225];
        for (i, j, value) in system.stiffness().entries() {
            actual[15 * i + j] = value;
        }
        let full_error = compare(&actual, &expected, 1e-12);
        let reduced = reduction(&r, case, &system);
        assert_eq!(reduced.free_dofs(), &[6, 7, 8]);
        let mut actual = vec![0.0; 9];
        for (i, j, v) in reduced.stiffness().entries() {
            actual[3 * i + j] = v;
        }
        compare(
            &actual,
            &case
                .reduced
                .iter()
                .flatten()
                .map(|x| fraction(x))
                .collect::<Vec<_>>(),
            1e-12,
        );
        compare(
            reduced.rhs(),
            &case.rhs.iter().map(|x| fraction(x)).collect::<Vec<_>>(),
            1e-12,
        );
        let solution = dense_solve(reduced.stiffness(), reduced.rhs());
        let expected_solution = internal_vector(&r, &system, &case.solution);
        let error = compare(
            &reduced.reconstruct(&solution).unwrap(),
            &expected_solution,
            1e-8,
        );
        compare(
            &reduced.residual(&solution).unwrap(),
            &internal_vector(&r, &system, &case.residual),
            1e-10,
        );
        let reactions = reduced.reactions(&solution).unwrap();
        assert_eq!(reactions.len(), 12);
        for (dof, reaction) in reactions {
            assert!((reaction - internal_vector(&r, &system, &case.residual)[dof]).abs() < 1e-10);
        }
        println!(
            "{}: dense stiffness {full_error:e}, displacement {error:e}",
            case.name
        );
    }
}

#[test]
fn multielement_affine_patch_reproduces_strain_stress_and_equilibrium() {
    let r = reference();
    let case = &r.cases[0];
    let (mesh, model) = setup(&r, case);
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let reduced = reduction(&r, case, &system);
    let solution = dense_solve(reduced.stiffness(), reduced.rhs());
    let full = reduced.reconstruct(&solution).unwrap();
    let a = r
        .gradient
        .each_ref()
        .map(|row| row.each_ref().map(|x| fraction(x)));
    let strain = [
        a[0][0],
        a[1][1],
        a[2][2],
        a[0][1] + a[1][0],
        a[1][2] + a[2][1],
        a[0][2] + a[2][0],
    ];
    let nu = fraction(&r.poisson);
    let mu = 150.0 / (2.0 * (1.0 + nu));
    let lambda = 150.0 * nu / ((1.0 + nu) * (1.0 - 2.0 * nu));
    let trace = strain[0] + strain[1] + strain[2];
    let stress = [
        lambda * trace + 2.0 * mu * strain[0],
        lambda * trace + 2.0 * mu * strain[1],
        lambda * trace + 2.0 * mu * strain[2],
        mu * strain[3],
        mu * strain[4],
        mu * strain[5],
    ];
    let law = IsotropicElastic::new(150.0, nu).unwrap();
    let mut energy = 0.0;
    let mut stress_error: f64 = 0.0;
    for (id, tet) in &mesh.mesh().tetrahedra {
        let u = ElementDisplacements::from_fn(|i, _| {
            full[system.dofs().index(&tet.nodes[i / 3], i % 3).unwrap()]
        });
        let response = Tet4Element::from_mesh(&mesh, *id)
            .unwrap()
            .response(&law, &u)
            .unwrap();
        compare(response.strain.as_slice(), &strain, 1e-8);
        stress_error = stress_error.max(compare(response.stress.as_slice(), &stress, 1e-8));
        energy += response.energy;
    }
    let ku = system.stiffness().multiply(&full).unwrap();
    assert!((energy - 0.5 * full.iter().zip(&ku).map(|(u, f)| u * f).sum::<f64>()).abs() < 1e-10);
    let residual = reduced.residual(&solution).unwrap();
    for i in reduced.free_dofs() {
        assert!(residual[*i].abs() < 1e-10);
    }
    let mut force = [0.0; 3];
    let mut moment = [0.0; 3];
    for node in system.dofs().nodes() {
        let x = mesh.mesh().nodes[node];
        let base = system.dofs().index(node, 0).unwrap();
        let f = &residual[base..base + 3];
        for i in 0..3 {
            force[i] += f[i];
            moment[i] += x[(i + 1) % 3] * f[(i + 2) % 3] - x[(i + 2) % 3] * f[(i + 1) % 3];
        }
    }
    assert!(norm(force) < 1e-10 && norm(moment) < 1e-10);
    println!("affine patch: stress {stress_error:e}");
}

#[test]
fn model_group_prescriptions_merge_and_nonzero_translation_is_recovered() {
    let r = reference();
    let (mesh, mut model) = setup(&r, &r.cases[0]);
    let condition = BoundaryCondition {
        node_group: GroupId("boundary".into()),
        components: PrescribedComponents {
            x: Some(0.2),
            y: Some(-0.3),
            z: Some(0.7),
        },
    };
    model.boundary_conditions = vec![condition.clone(), condition];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    assert_eq!(system.prescriptions().len(), 12);
    let reduced = system.reduce(&[0.0; 15]).unwrap();
    let solution = dense_solve(reduced.stiffness(), reduced.rhs());
    compare(
        &reduced.reconstruct(&solution).unwrap(),
        &[0.2, -0.3, 0.7].repeat(5),
        1e-10,
    );
    assert!(norm(reduced.residual(&solution).unwrap()) < 1e-10);
    model.boundary_conditions[1].components.x = Some(0.21);
    assert!(AssembledSystem::assemble(&mesh, &model).is_err());
}

#[test]
fn missing_duplicate_and_invalid_materials_or_orphan_nodes_are_rejected() {
    let r = reference();
    let (mesh, model) = setup(&r, &r.cases[0]);
    let mut missing = model.clone();
    missing.material_assignments.pop();
    assert!(AssembledSystem::assemble(&mesh, &missing).is_err());
    let mut duplicate = model.clone();
    duplicate
        .material_assignments
        .push(duplicate.material_assignments[0].clone());
    assert!(AssembledSystem::assemble(&mesh, &duplicate).is_err());
    let mut invalid = model.clone();
    invalid.materials[0] = Material::IsotropicLinearElastic {
        id: MaterialId("m0".into()),
        young: f64::NAN,
        poisson: 0.25,
    };
    assert!(AssembledSystem::assemble(&mesh, &invalid).is_err());
    // An extra imported region not declared by the model must still be covered.
    let mut missing = model.clone();
    missing.mesh.groups.volumes.pop();
    missing.material_assignments.pop();
    assert!(AssembledSystem::assemble(&mesh, &missing).is_err());
    let mut orphan = mesh.mesh().clone();
    orphan.nodes.insert(NodeId(9999), [8.0, 8.0, 8.0]);
    assert!(orphan.into_validated().is_err());
    let mut near = model.clone();
    near.materials[0] = Material::IsotropicLinearElastic {
        id: MaterialId("m0".into()),
        young: 150.0,
        poisson: 0.49,
    };
    let advisory = AssembledSystem::assemble(&mesh, &near).unwrap();
    assert_eq!(advisory.material_warnings().len(), 1);
    assert_eq!(advisory.material_warnings()[0].0, MaterialId("m0".into()));
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    assert!(system.dofs().index(&NodeId(0), 0).is_err());
    assert!(system.dofs().index(&NodeId(7), 3).is_err());
}

#[test]
fn elimination_handles_partial_components_and_preserves_original_forces() {
    let k = SparseMatrix::from_triplets(
        3,
        [
            (0, 0, 4.0),
            (0, 1, -1.0),
            (1, 0, -1.0),
            (1, 1, 3.0),
            (1, 2, -1.0),
            (2, 1, -1.0),
            (2, 2, 2.0),
        ],
    )
    .unwrap();
    let reduced = ReducedSystem::new(
        &k,
        &[7.0, 1.0, 8.0],
        &[PrescribedDof { dof: 0, value: 2.0 }],
    )
    .unwrap();
    assert_eq!(reduced.rhs(), &[3.0, 8.0]);
    assert_eq!(reduced.free_dofs(), &[1, 2]);
    // Exact solution: u1=14/5, u2=27/5; reaction includes -F0.
    let u = dense_solve(reduced.stiffness(), reduced.rhs());
    compare(&u, &[2.8, 5.4], 1e-12);
    compare(&reduced.residual(&u).unwrap(), &[-1.8, 0.0, 0.0], 1e-12);
    assert_eq!(k.entries().count(), 7);
    let zero = ReducedSystem::new(
        &k,
        &[7.0, 1.0, 8.0],
        &[PrescribedDof { dof: 0, value: 0.0 }],
    )
    .unwrap();
    assert_eq!(zero.rhs(), &[1.0, 8.0]);
}

#[test]
fn fully_prescribed_unconstrained_and_empty_reductions_are_defined() {
    let k = SparseMatrix::from_triplets(2, [(0, 0, 3.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 2.0)])
        .unwrap();
    let unconstrained = ReducedSystem::new(&k, &[1.0, 2.0], &[]).unwrap();
    assert_eq!(unconstrained.rhs(), &[1.0, 2.0]);
    assert_eq!(unconstrained.reconstruct(&[0.0, 1.0]).unwrap(), [0.0, 1.0]);
    let all = ReducedSystem::new(
        &k,
        &[1.0, 2.0],
        &[
            PrescribedDof {
                dof: 1,
                value: -1.0,
            },
            PrescribedDof { dof: 0, value: 2.0 },
        ],
    )
    .unwrap();
    assert_eq!(all.stiffness().size(), 0);
    assert!(all.rhs().is_empty());
    assert_eq!(all.residual(&[]).unwrap(), [4.0, -2.0]);
    let empty = ReducedSystem::new(&SparseMatrix::from_triplets(0, []).unwrap(), &[], &[]).unwrap();
    assert!(empty.reconstruct(&[]).unwrap().is_empty());
}

#[test]
fn invalid_constraints_vectors_and_reduction_overflow_are_rejected() {
    let k = SparseMatrix::from_triplets(2, [(0, 0, 2.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 2.0)])
        .unwrap();
    for constraints in [
        vec![PrescribedDof { dof: 2, value: 0.0 }],
        vec![PrescribedDof {
            dof: 0,
            value: f64::NAN,
        }],
        vec![
            PrescribedDof { dof: 0, value: 0.0 },
            PrescribedDof { dof: 0, value: 1.0 },
        ],
    ] {
        assert!(ReducedSystem::new(&k, &[0.0; 2], &constraints).is_err());
    }
    assert!(ReducedSystem::new(&k, &[0.0], &[]).is_err());
    assert!(ReducedSystem::new(&k, &[f64::INFINITY, 0.0], &[]).is_err());
    let reduced = ReducedSystem::new(
        &k,
        &[0.0; 2],
        &[
            PrescribedDof { dof: 0, value: 1.0 },
            PrescribedDof { dof: 0, value: 1.0 },
        ],
    )
    .unwrap();
    assert!(reduced.reconstruct(&[]).is_err());
    assert!(reduced.residual(&[f64::NAN]).is_err());
    let huge = SparseMatrix::from_triplets(2, [(0, 1, f64::MAX), (1, 0, f64::MAX)]).unwrap();
    assert!(ReducedSystem::new(&huge, &[0.0; 2], &[PrescribedDof { dof: 0, value: 2.0 }]).is_err());
    let huge = ReducedSystem::new(
        &huge,
        &[f64::MAX, 0.0],
        &[
            PrescribedDof { dof: 0, value: 0.0 },
            PrescribedDof {
                dof: 1,
                value: -1.0,
            },
        ],
    )
    .unwrap();
    assert!(huge.residual(&[]).is_err());
}

#[test]
fn partial_model_components_preserve_free_axes_and_explicit_zero() {
    let r = reference();
    let (mesh, mut model) = setup(&r, &r.cases[0]);
    model.boundary_conditions = vec![BoundaryCondition {
        node_group: GroupId("boundary".into()),
        components: PrescribedComponents {
            x: Some(0.0),
            y: None,
            z: Some(0.3),
        },
    }];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    assert_eq!(system.prescriptions().len(), 8);
    for p in system.prescriptions() {
        assert_ne!(p.dof % 3, 1);
    }
    let reduced = system.reduce(&[0.0; 15]).unwrap();
    assert_eq!(reduced.free_dofs().len(), 7);
    for node in system.dofs().nodes() {
        assert!(
            reduced
                .free_dofs()
                .contains(&system.dofs().index(node, 1).unwrap())
        );
    }
}

#[test]
fn global_symmetry_and_six_rigid_modes_survive_heterogeneous_assembly() {
    let r = reference();
    let (mesh, mut model) = setup(&r, &r.cases[1]);
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    model.material_assignments.reverse();
    model.materials.reverse();
    let reordered = AssembledSystem::assemble(&mesh, &model).unwrap();
    assert_eq!(
        system.stiffness().entries().collect::<Vec<_>>(),
        reordered.stiffness().entries().collect::<Vec<_>>()
    );
    let mut dense = DMatrix::<f64>::zeros(15, 15);
    for (i, j, v) in system.stiffness().entries() {
        dense[(i, j)] = v;
    }
    assert!((&dense - dense.transpose()).norm() / dense.norm() < 1e-10);
    for mode in 0..6 {
        let mut u = vec![0.0; 15];
        for node in system.dofs().nodes() {
            let x = mesh.mesh().nodes[node];
            let base = system.dofs().index(node, 0).unwrap();
            if mode < 3 {
                u[base + mode] = 1.0;
            } else {
                let axis = mode - 3;
                u[base + (axis + 1) % 3] = -x[(axis + 2) % 3];
                u[base + (axis + 2) % 3] = x[(axis + 1) % 3];
            }
        }
        assert!(norm(system.stiffness().multiply(&u).unwrap()) / (dense.norm() * norm(u)) < 1e-10);
    }
}

#[test]
fn cg_jacobi_reduced_solutions_and_reactions_match_exact_references() {
    let r = reference();
    for case in &r.cases {
        let (mesh, model) = setup(&r, case);
        let system = AssembledSystem::assemble(&mesh, &model).unwrap();
        let reduced = reduction(&r, case, &system);
        let mut options = model.solver_settings.clone();
        options.rtol = 1e-12;
        options.atol = 1e-14;
        let report = reduced.solve(&options).unwrap();
        let expected = internal_vector(&r, &system, &case.solution);
        let error = compare(
            &reduced.reconstruct(&report.solution).unwrap(),
            &expected,
            1e-8,
        );
        compare(
            &reduced.residual(&report.solution).unwrap(),
            &internal_vector(&r, &system, &case.residual),
            1e-10,
        );
        println!(
            "CG {}: displacement {error:e}, iterations {}, relative residual {:?}",
            case.name, report.iterations, report.relative_residual
        );
        assert!(report.residual_norm <= report.threshold);
    }
}

#[test]
fn cg_handles_fully_prescribed_system_and_rejects_unbalanced_free_body() {
    let r = reference();
    let (mesh, model) = setup(&r, &r.cases[0]);
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let prescriptions: Vec<_> = (0..15)
        .map(|dof| PrescribedDof { dof, value: 0.1 })
        .collect();
    let all = ReducedSystem::new(system.stiffness(), &[0.0; 15], &prescriptions).unwrap();
    let report = all.solve(&model.solver_settings).unwrap();
    assert_eq!(report.iterations, 0);
    assert!(report.solution.is_empty());
    assert_eq!(all.reconstruct(&report.solution).unwrap(), vec![0.1; 15]);
    let mut force = [0.0; 15];
    for dof in (0..15).step_by(3) {
        force[dof] = 1.0;
    }
    let free = system.reduce(&force).unwrap();
    assert!(free.solve(&model.solver_settings).is_err());
}
