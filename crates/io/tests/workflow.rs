//! Independent file workflow defaults, option validation and pre-solve range failures.
use ferrasim_io::{
    RunOptions, SimulationRunError, load_meshed_model, model_to_json, run_simulation_file,
    validate_simulation_file,
};
use ferrasim_model::{Material, MaterialId};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ferrasim workflow {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn input() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/uniaxial/model.json")
}
#[test]
fn automatic_scales_are_recorded_and_validation_creates_no_bundle() {
    let temp = Temp::new();
    let summary = validate_simulation_file(input()).unwrap();
    assert_eq!((summary.nodes, summary.elements), (8, 6));
    assert!(std::fs::read_dir(&temp.0).unwrap().next().is_none());
    let r = run_simulation_file(input(), temp.0.join("results"), &RunOptions::default()).unwrap();
    assert!((r.results.balance().scales.force() - 10.0).abs() < 1e-12);
    assert!((r.results.balance().scales.length() - 6.0_f64.sqrt()).abs() < 1e-12);
}
#[test]
fn invalid_api_options_fail_before_input_or_output_operations() {
    let temp = Temp::new();
    for options in [
        RunOptions {
            force_scale: Some(0.0),
            ..Default::default()
        },
        RunOptions {
            length_scale: Some(f64::NAN),
            ..Default::default()
        },
        RunOptions {
            origin: [f64::INFINITY, 0.0, 0.0],
            ..Default::default()
        },
    ] {
        let error = run_simulation_file("absent", temp.0.join("none"), &options).unwrap_err();
        assert!(matches!(error, SimulationRunError::Options(_)));
        assert_eq!(error.exit_code(), 2);
        assert!(!temp.0.join("none").exists());
    }
}
#[test]
fn unrepresentable_material_is_preparation_error_before_solver_or_output() {
    let temp = Temp::new();
    let loaded = load_meshed_model(input()).unwrap();
    let mut model = loaded.model.simulation;
    model.materials = vec![Material::IsotropicLinearElastic {
        id: MaterialId("elastic".into()),
        young: f64::MAX,
        poisson: 0.25,
    }];
    std::fs::write(temp.0.join("model.json"), model_to_json(&model).unwrap()).unwrap();
    std::fs::copy(loaded.model.mesh_path, temp.0.join("mesh.msh")).unwrap();
    let error = run_simulation_file(
        temp.0.join("model.json"),
        temp.0.join("none"),
        &RunOptions::default(),
    )
    .unwrap_err();
    assert!(matches!(error, SimulationRunError::Preparation(_)));
    assert_eq!(error.exit_code(), 3);
    assert!(!temp.0.join("none").exists());
}
