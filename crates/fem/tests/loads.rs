//! Model load integration, analytic uniaxial reference and global balance diagnostics.
use ferrasim_fem::{
    AssembledSystem, BalanceScales, ElementDisplacements, IsotropicElastic, Tet4Element,
};
use ferrasim_mesh::{ElementId, PhysicalGroup, Triangle, ValidatedMesh};
use ferrasim_model::{
    BoundaryCondition, GroupId, Load, NodeId, PrescribedComponents, Simulation, Vector3,
};
use serde::Deserialize;
#[derive(Deserialize)]
struct Reference {
    node_ids: Vec<u64>,
    forces: Vec<[String; 3]>,
    displacements: Vec<[String; 3]>,
    stress: [String; 6],
    strain_energy: String,
    resultant: [String; 3],
    moment: [String; 3],
}
fn reference() -> Reference {
    serde_json::from_str(include_str!(
        "../../../validation/fixtures/uniaxial-reference.json"
    ))
    .unwrap()
}
fn fraction(s: &str) -> f64 {
    match s.split_once('/') {
        Some((a, b)) => a.parse::<f64>().unwrap() / b.parse::<f64>().unwrap(),
        None => s.parse().unwrap(),
    }
}
fn setup() -> (ValidatedMesh, Simulation) {
    (
        ferrasim_io::parse_msh22(include_str!("../../../examples/uniaxial/mesh.msh")).unwrap(),
        serde_json::from_str(include_str!("../../../examples/uniaxial/model.json")).unwrap(),
    )
}
fn vector(v: [f64; 3]) -> Vector3 {
    Vector3 {
        x: v[0],
        y: v[1],
        z: v[2],
    }
}
fn surface(name: &str, v: [f64; 3]) -> Load {
    Load::SurfaceTraction {
        surface_group: GroupId(name.into()),
        components: vector(v),
    }
}
fn nodal(id: u64, v: [f64; 3]) -> Load {
    Load::NodalForce {
        node: NodeId(id),
        components: vector(v),
    }
}
fn norm(v: impl IntoIterator<Item = f64>) -> f64 {
    v.into_iter().fold(0.0_f64, |n, v| n.hypot(v))
}
fn compare(a: &[f64], b: &[f64], scale: f64, tolerance: f64) -> f64 {
    assert_eq!(a.len(), b.len());
    let error = norm(a.iter().zip(b).map(|(a, b)| a - b)) / norm(b.iter().copied()).max(scale);
    assert!(error <= tolerance, "error {error:e}");
    error
}
fn expected(system: &AssembledSystem, ids: &[u64], values: &[[String; 3]]) -> Vec<f64> {
    let mut v = vec![0.0; system.dofs().len()];
    for (id, xyz) in ids.iter().zip(values) {
        for axis in 0..3 {
            v[system.dofs().index(&NodeId(*id), axis).unwrap()] = fraction(&xyz[axis]);
        }
    }
    v
}
fn scales() -> BalanceScales {
    BalanceScales::new(10.0, 2.0).unwrap()
}
fn balanced(balance: &ferrasim_fem::MechanicalBalance) {
    println!(
        "balance force {:e}, moment {:e}, energy {:e}",
        balance.relative_force_error, balance.relative_moment_error, balance.relative_energy_error
    );
    assert!(balance.relative_prescription_error <= 1e-10);
    assert!(balance.relative_force_error <= 1e-6);
    assert!(balance.relative_moment_error <= 1e-6);
    assert!(balance.relative_free_residual <= 1e-6);
    assert!(balance.relative_energy_error <= 1e-10);
}

#[test]
fn surface_forces_match_exact_reference_without_shared_node_duplication() {
    let (mesh, model) = setup();
    let r = reference();
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let error = compare(
        system.forces(),
        &expected(&system, &r.node_ids, &r.forces),
        1.0,
        1e-10,
    );
    let balance = system.balance(&[0.0; 24], [0.0; 3], scales()).unwrap();
    compare(
        &balance.external_force,
        &r.resultant.map(|s| fraction(&s)),
        1.0,
        1e-10,
    );
    compare(
        &balance.external_moment,
        &r.moment.map(|s| fraction(&s)),
        1.0,
        1e-10,
    );
    let mut reversed = mesh.mesh().clone();
    for tri in reversed.triangles.values_mut() {
        tri.nodes.swap(0, 1);
    }
    let other = AssembledSystem::assemble(&reversed.into_validated().unwrap(), &model).unwrap();
    assert_eq!(system.forces(), other.forces());
    println!("surface relative load-vector error {error:e}");
}
#[test]
fn nodal_records_and_surface_records_superpose_once_per_explicit_entry() {
    let (mesh, mut model) = setup();
    model.loads = vec![
        nodal(90, [1.0, 2.0, 3.0]),
        nodal(90, [4.0, -2.0, 1.0]),
        nodal(7, [-1.0, 0.0, 0.0]),
    ];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let mut wanted = vec![0.0; 24];
    let i = system.dofs().index(&NodeId(90), 0).unwrap();
    wanted[i..i + 3].copy_from_slice(&[5.0, 0.0, 4.0]);
    wanted[system.dofs().index(&NodeId(7), 0).unwrap()] = -1.0;
    assert_eq!(system.forces(), wanted);
    model.loads = vec![surface("x1", [2.0, 3.0, -4.0])];
    let single = AssembledSystem::assemble(&mesh, &model).unwrap();
    model.loads.push(model.loads[0].clone());
    let double = AssembledSystem::assemble(&mesh, &model).unwrap();
    compare(
        double.forces(),
        &single.forces().iter().map(|f| 2.0 * f).collect::<Vec<_>>(),
        1.0,
        1e-14,
    );
}
#[test]
fn oblique_face_integrates_global_vector_traction_with_geometric_area() {
    let mut raw = ferrasim_io::parse_msh22(include_str!(
        "../../../validation/fixtures/tet4-positive.msh"
    ))
    .unwrap()
    .mesh()
    .clone();
    let id = ElementId(301);
    raw.triangles.insert(
        id,
        Triangle {
            id,
            nodes: [NodeId(2), NodeId(3), NodeId(4)],
            physical_tag: Some(9),
            entity_tag: None,
        },
    );
    raw.physical_groups.push(PhysicalGroup {
        dimension: 2,
        tag: 9,
        name: GroupId("oblique".into()),
    });
    let mesh = raw.into_validated().unwrap();
    let mut model: Simulation =
        serde_json::from_str(include_str!("../../../examples/minimal/model.json")).unwrap();
    model.mesh.groups.surfaces.push(GroupId("oblique".into()));
    model.loads = vec![surface("oblique", [2.0, -3.0, 4.0])];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let area = 3.0_f64.sqrt() / 2.0;
    let b = system
        .balance(&[0.0; 12], [0.0; 3], BalanceScales::new(10.0, 1.0).unwrap())
        .unwrap();
    compare(
        &b.external_force,
        &[2.0 * area, -3.0 * area, 4.0 * area],
        1.0,
        1e-10,
    );
    compare(
        &b.external_moment,
        &[7.0 * area / 3.0, -2.0 * area / 3.0, -5.0 * area / 3.0],
        1.0,
        1e-10,
    );
    for id in [2, 3, 4] {
        let i = system.dofs().index(&NodeId(id), 0).unwrap();
        compare(
            &system.forces()[i..i + 3],
            &[2.0 * area / 3.0, -area, 4.0 * area / 3.0],
            1.0,
            1e-10,
        );
    }
}
#[test]
fn uniaxial_traction_matches_analytic_displacements_stress_and_energy() {
    let (mesh, model) = setup();
    let r = reference();
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let solution = system
        .solve_model_loads(&model.solver_settings, [0.0; 3], scales())
        .unwrap();
    balanced(&solution.balance);
    let displacement_error = compare(
        &solution.displacements,
        &expected(&system, &r.node_ids, &r.displacements),
        1e-3,
        1e-6,
    );
    let axial_nodes: Vec<_> = system
        .coordinates()
        .iter()
        .enumerate()
        .filter(|(_, x)| x[0] == 2.0)
        .map(|(i, _)| solution.displacements[3 * i])
        .collect();
    let axial_average = axial_nodes.iter().sum::<f64>() / axial_nodes.len() as f64;
    assert!(((axial_average - 0.02) / 0.02).abs() <= 1e-6);
    let law = IsotropicElastic::new(1000.0, 0.25).unwrap();
    let stress = r.stress.map(|s| fraction(&s));
    let mut stress_error: f64 = 0.0;
    let mut energy = 0.0;
    for (id, tet) in &mesh.mesh().tetrahedra {
        let u = ElementDisplacements::from_fn(|i, _| {
            solution.displacements[system.dofs().index(&tet.nodes[i / 3], i % 3).unwrap()]
        });
        let response = Tet4Element::from_mesh(&mesh, *id)
            .unwrap()
            .response(&law, &u)
            .unwrap();
        stress_error = stress_error.max(compare(response.stress.as_slice(), &stress, 1.0, 1e-6));
        energy += response.energy;
    }
    assert!((energy - fraction(&r.strain_energy)).abs() < 1e-10);
    assert!((energy - solution.balance.strain_energy).abs() < 1e-10);
    println!(
        "uniaxial displacement {displacement_error:e}, stress {stress_error:e}, force {:e}, moment {:e}, energy {:e}",
        solution.balance.relative_force_error,
        solution.balance.relative_moment_error,
        solution.balance.relative_energy_error
    );
}
#[test]
fn mixed_loads_and_nonzero_prescriptions_include_reaction_work() {
    let (mesh, mut model) = setup();
    for bc in &mut model.boundary_conditions {
        match bc.node_group.0.as_str() {
            "x0" => bc.components.x = Some(0.03),
            "y0" => bc.components.y = Some(-0.01),
            _ => bc.components.z = Some(0.02),
        }
    }
    model.loads.push(nodal(90, [2.0, -3.0, 4.0]));
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let solution = system
        .solve_model_loads(&model.solver_settings, [0.2, -0.5, 0.8], scales())
        .unwrap();
    balanced(&solution.balance);
    compare(
        &solution.balance.external_force,
        &[12.0, -3.0, 4.0],
        1.0,
        1e-10,
    );
    assert!((solution.balance.strain_energy - 0.1).abs() < 1e-10);
    assert!((2.0 * solution.balance.strain_energy - solution.balance.load_work).abs() > 0.1);
    assert!(solution.balance.prescription_work.abs() > 0.1);
    println!(
        "mixed force {:e}, moment {:e}, energy {:e}",
        solution.balance.relative_force_error,
        solution.balance.relative_moment_error,
        solution.balance.relative_energy_error
    );
}
#[test]
fn force_couple_and_zero_load_have_safe_normalization_and_origin_covariance() {
    let (mesh, mut model) = setup();
    model.loads = vec![nodal(400, [0.0, 1.0, 0.0]), nodal(23, [0.0, -1.0, 0.0])];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let solution = system
        .solve_model_loads(&model.solver_settings, [0.0; 3], scales())
        .unwrap();
    balanced(&solution.balance);
    compare(&solution.balance.external_force, &[0.0; 3], 1.0, 1e-10);
    compare(
        &solution.balance.external_moment,
        &[0.0, 0.0, 2.0],
        1.0,
        1e-10,
    );
    let shifted = system
        .balance(&solution.displacements, [3.0, -2.0, 1.0], scales())
        .unwrap();
    balanced(&shifted);
    compare(
        &shifted.external_moment,
        &solution.balance.external_moment,
        1.0,
        1e-10,
    );
    model.loads.clear();
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let zero = system
        .solve_model_loads(&model.solver_settings, [0.0; 3], scales())
        .unwrap();
    assert_eq!(zero.solver.iterations, 0);
    balanced(&zero.balance);
    assert_eq!(zero.balance.strain_energy, 0.0);
}
#[test]
fn invalid_loaded_surfaces_nodes_and_nonfinite_loads_are_rejected() {
    let (mesh, model) = setup();
    for load in [
        surface("missing", [1.0, 0.0, 0.0]),
        nodal(999, [1.0, 0.0, 0.0]),
        surface("x1", [f64::NAN, 0.0, 0.0]),
        nodal(7, [0.0, f64::INFINITY, 0.0]),
    ] {
        let mut invalid = model.clone();
        invalid.loads = vec![load];
        assert!(AssembledSystem::assemble(&mesh, &invalid).is_err());
    }
    let mut raw = mesh.mesh().clone();
    raw.physical_groups.push(PhysicalGroup {
        dimension: 2,
        tag: 7,
        name: GroupId("empty".into()),
    });
    let empty = raw.into_validated().unwrap();
    let mut invalid = model.clone();
    invalid.mesh.groups.surfaces.push(GroupId("empty".into()));
    invalid.loads = vec![surface("empty", [1.0, 0.0, 0.0])];
    assert!(AssembledSystem::assemble(&empty, &invalid).is_err());
    let internal = ferrasim_io::parse_msh22(include_str!(
        "../../../validation/fixtures/two-tets-interface.msh"
    ))
    .unwrap();
    let mut invalid = model;
    invalid.boundary_conditions.clear();
    invalid.mesh.groups.nodes.clear();
    invalid.mesh.groups.surfaces = vec![GroupId("interface".into())];
    invalid.loads = vec![surface("interface", [1.0, 0.0, 0.0])];
    assert!(AssembledSystem::assemble(&internal, &invalid).is_err());
}
#[test]
fn unrepresentable_load_accumulation_and_traction_are_explicit_errors() {
    let (mesh, mut model) = setup();
    model.loads = vec![nodal(7, [f64::MAX, 0.0, 0.0]); 2];
    assert!(AssembledSystem::assemble(&mesh, &model).is_err());
    model.loads = vec![surface("x1", [f64::from_bits(1), 0.0, 0.0])];
    assert!(AssembledSystem::assemble(&mesh, &model).is_err());
    let mut large = mesh.mesh().clone();
    for p in large.nodes.values_mut() {
        for x in p {
            *x *= 100.0;
        }
    }
    model.loads = vec![surface("x1", [f64::MAX, 0.0, 0.0])];
    assert!(AssembledSystem::assemble(&large.into_validated().unwrap(), &model).is_err());
}
#[test]
fn invalid_balance_inputs_fail_and_an_unsolved_field_shows_imbalance() {
    for (f, l) in [
        (0.0, 1.0),
        (-1.0, 1.0),
        (1.0, f64::NAN),
        (f64::MAX, 2.0),
        (1e-300, 1e-300),
    ] {
        assert!(BalanceScales::new(f, l).is_err());
    }
    let (mesh, model) = setup();
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    assert!(system.balance(&[], [0.0; 3], scales()).is_err());
    assert!(system.balance(&[f64::NAN; 24], [0.0; 3], scales()).is_err());
    assert!(
        system
            .balance(&[0.0; 24], [f64::INFINITY, 0.0, 0.0], scales())
            .is_err()
    );
    assert!(system.balance(&[f64::MAX; 24], [0.0; 3], scales()).is_err());
    let mut limited = model.solver_settings.clone();
    limited.max_iterations = 1;
    assert!(matches!(
        system.solve_model_loads(&limited, [0.0; 3], scales()),
        Err(ferrasim_fem::FemError::Solver(_))
    ));
    let wrong = system.balance(&[0.0; 24], [0.0; 3], scales()).unwrap();
    assert!(wrong.relative_force_error > 1e-6 && wrong.relative_free_residual > 1e-6);
    let mut shifted_model = model.clone();
    shifted_model.boundary_conditions[0].components.x = Some(0.1);
    let shifted = AssembledSystem::assemble(&mesh, &shifted_model).unwrap();
    assert!(
        shifted
            .balance(&[0.0; 24], [0.0; 3], scales())
            .unwrap()
            .relative_prescription_error
            > 0.0
    );
}
#[test]
fn fully_prescribed_rigid_translation_includes_original_forces_in_reactions() {
    let (mesh, mut model) = setup();
    model.mesh.groups.nodes = model.mesh.groups.surfaces.clone();
    model.boundary_conditions = model
        .mesh
        .groups
        .nodes
        .iter()
        .map(|g| BoundaryCondition {
            node_group: g.clone(),
            components: PrescribedComponents {
                x: Some(0.1),
                y: Some(0.2),
                z: Some(0.3),
            },
        })
        .collect();
    model.loads = vec![nodal(90, [2.0, 3.0, -1.0])];
    let system = AssembledSystem::assemble(&mesh, &model).unwrap();
    let solution = system
        .solve_model_loads(&model.solver_settings, [0.0; 3], scales())
        .unwrap();
    assert_eq!(solution.solver.iterations, 0);
    balanced(&solution.balance);
    compare(
        &solution.balance.reaction_force,
        &[-2.0, -3.0, 1.0],
        1.0,
        1e-10,
    );
    assert!(solution.balance.strain_energy.abs() < 1e-10);
}
