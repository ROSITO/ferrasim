//! Analytic field recovery, equivalent stress and failure-before-results contracts.
use ferrasim_fem::{BalanceScales, solve_static_results, von_mises};
use ferrasim_model::{Load, Material, MaterialId, NodeId, Vector3};
fn inputs() -> ferrasim_io::MeshedSimulation {
    ferrasim_io::load_meshed_model(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/uniaxial/model.json"),
    )
    .unwrap()
}
#[test]
fn von_mises_matches_uniaxial_hydrostatic_and_all_shear_planes_at_multiple_scales() {
    for scale in [1e-200, 1.0, 1e200] {
        assert!(
            (von_mises(&[scale, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap() / scale - 1.0).abs() < 1e-14
        );
        assert_eq!(
            von_mises(&[scale, scale, scale, 0.0, 0.0, 0.0]).unwrap(),
            0.0
        );
        for axis in 3..6 {
            let mut stress = [0.0; 6];
            stress[axis] = scale;
            assert!((von_mises(&stress).unwrap() / scale - 3.0_f64.sqrt()).abs() < 1e-14);
        }
    }
    assert_eq!(von_mises(&[0.0; 6]).unwrap(), 0.0);
    assert_eq!(
        von_mises(&[f64::MAX, f64::MAX, f64::MAX, 0.0, 0.0, 0.0]).unwrap(),
        0.0
    );
}
#[test]
fn invalid_or_overflowing_equivalent_stress_is_rejected() {
    for stress in [
        [f64::NAN; 6],
        [f64::INFINITY; 6],
        [0.0, 0.0, 0.0, f64::MAX, 0.0, 0.0],
    ] {
        assert!(von_mises(&stress).is_err());
    }
}
#[test]
fn recovered_fields_match_uniaxial_reference_and_keep_reactions_masked() {
    let loaded = inputs();
    let model = &loaded.model.simulation;
    let results = solve_static_results(
        &loaded.mesh,
        model,
        [0.0; 3],
        BalanceScales::new(10.0, 2.0).unwrap(),
    )
    .unwrap();
    assert_eq!(results.nodes().len(), 8);
    assert_eq!(results.elements().len(), 6);
    let mut energy = 0.0;
    for n in results.nodes() {
        let expected = [
            0.01 * n.coordinates[0],
            -0.0025 * n.coordinates[1],
            -0.0025 * n.coordinates[2],
        ];
        for (actual, expected) in n.displacement.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-12);
        }
        for axis in 0..3 {
            assert_eq!(n.reaction[axis].is_some(), n.prescribed[axis].is_some());
        }
    }
    for e in results.elements() {
        assert!((e.stress[0] - 10.0).abs() < 1e-10);
        assert!(e.stress[1..].iter().all(|x| x.abs() < 1e-10));
        assert!((e.von_mises - 10.0).abs() < 1e-10);
        assert_eq!(e.material.0, "elastic");
        energy += e.energy;
    }
    assert!((energy - results.balance().strain_energy).abs() < 1e-12);
    assert_eq!(results.model(), model);
    assert_eq!(results.mesh().mesh().tetrahedra.len(), 6);
}
#[test]
fn pure_shear_recovery_preserves_engineering_gamma_and_voigt_order() {
    let mut loaded = ferrasim_io::load_meshed_model(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/minimal/model.json"),
    )
    .unwrap();
    let model = &mut loaded.model.simulation;
    model.materials = vec![Material::IsotropicLinearElastic {
        id: MaterialId("steel".into()),
        young: 6.0,
        poisson: 0.0,
    }];
    model.loads = vec![Load::NodalForce {
        node: NodeId(4),
        components: Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
    }];
    let r = solve_static_results(
        &loaded.mesh,
        model,
        [0.0; 3],
        BalanceScales::new(1.0, 1.0).unwrap(),
    )
    .unwrap();
    let e = &r.elements()[0];
    assert!((e.strain[5] - 2.0).abs() < 1e-12);
    assert!((e.stress[5] - 6.0).abs() < 1e-12);
    assert!(
        e.strain[..5]
            .iter()
            .chain(&e.stress[..5])
            .all(|x| x.abs() < 1e-12)
    );
    assert!((e.von_mises - 6.0 * 3.0_f64.sqrt()).abs() < 1e-12);
}
#[test]
fn failed_solve_or_invalid_model_never_produces_result_fields() {
    let mut loaded = inputs();
    loaded.model.simulation.solver_settings.max_iterations = 1;
    assert!(
        solve_static_results(
            &loaded.mesh,
            &loaded.model.simulation,
            [0.0; 3],
            BalanceScales::new(10.0, 2.0).unwrap()
        )
        .is_err()
    );
    loaded.model.simulation.material_assignments.clear();
    assert!(
        solve_static_results(
            &loaded.mesh,
            &loaded.model.simulation,
            [0.0; 3],
            BalanceScales::new(10.0, 2.0).unwrap()
        )
        .is_err()
    );
}
