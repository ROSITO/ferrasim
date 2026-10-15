//! Native CLI contracts: paths, API parity, stage-specific failures and output preservation.
use ferrasim_io::{RunOptions, load_meshed_model, model_to_json, run_simulation_file};
use ferrasim_model::{Load, NodeId, Vector3};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    model: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "FerraSim S08 é space {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/uniaxial");
        std::fs::copy(source.join("mesh.msh"), root.join("mesh.msh")).unwrap();
        let model = root.join("modèle source.json");
        std::fs::copy(source.join("model.json"), &model).unwrap();
        Self { root, model }
    }
    fn change(&self, edit: impl FnOnce(&mut ferrasim_model::Simulation)) {
        let mut model = load_meshed_model(&self.model).unwrap().model.simulation;
        edit(&mut model);
        std::fs::write(&self.model, model_to_json(&model).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
fn cli(cwd: &Path, args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ferrasim"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}
fn failed(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
#[test]
fn help_version_and_usage_are_explicit() {
    let f = Fixture::new();
    for args in [
        vec![],
        vec!["--help"],
        vec!["validate", "--help"],
        vec!["solve", "--help"],
        vec!["--version"],
    ] {
        let o = cli(
            &f.root,
            &args.iter().map(std::ffi::OsStr::new).collect::<Vec<_>>(),
        );
        assert!(o.status.success());
        assert!(!o.stdout.is_empty());
    }
    for args in [
        vec!["unknown"],
        vec!["solve"],
        vec!["solve", "a.json"],
        vec!["validate", "--output", "x"],
        vec!["validate", "--unknown"],
        vec!["solve", "a", "--output", "x", "--force-scale", "NaN"],
        vec!["solve", "a", "--output", "x", "--force-scale", "-1"],
        vec!["solve", "a", "--output", "x", "--origin", "0"],
    ] {
        failed(
            &cli(
                &f.root,
                &args.iter().map(std::ffi::OsStr::new).collect::<Vec<_>>(),
            ),
            2,
        );
    }
}
#[test]
fn validation_reads_mesh_relative_to_model_without_creating_outputs() {
    let f = Fixture::new();
    let other = f.root.join("other cwd");
    std::fs::create_dir(&other).unwrap();
    let before = std::fs::read_dir(&f.root).unwrap().count();
    let o = cli(&other, &["validate".as_ref(), f.model.as_os_str()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("8 nodes, 6 TET4"));
    assert_eq!(std::fs::read_dir(&f.root).unwrap().count(), before);
}
#[test]
fn cli_and_api_produce_identical_fields_reports_and_manifest() {
    let f = Fixture::new();
    let cli_dir = f.root.join("résultats CLI");
    let api_dir = f.root.join("résultats API");
    let o = cli(
        &f.root,
        &[
            "solve".as_ref(),
            f.model.as_os_str(),
            "--output".as_ref(),
            cli_dir.as_os_str(),
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stdout).contains("Complete:"));
    let run = run_simulation_file(&f.model, &api_dir, &RunOptions::default()).unwrap();
    for file in run.exported.files {
        assert_eq!(
            std::fs::read(cli_dir.join(&file)).unwrap(),
            std::fs::read(api_dir.join(file)).unwrap()
        );
    }
}
#[test]
fn explicit_scales_origin_and_options_before_model_work() {
    let f = Fixture::new();
    let out = f.root.join("explicit");
    let o = cli(
        &f.root,
        &[
            "solve".as_ref(),
            "--output".as_ref(),
            out.as_os_str(),
            "--force-scale".as_ref(),
            "10".as_ref(),
            "--length-scale".as_ref(),
            "2".as_ref(),
            "--origin".as_ref(),
            "1".as_ref(),
            "-2".as_ref(),
            "3".as_ref(),
            f.model.as_os_str(),
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let api = f.root.join("explicit-api");
    let r = run_simulation_file(
        &f.model,
        &api,
        &RunOptions {
            force_scale: Some(10.0),
            length_scale: Some(2.0),
            origin: [1.0, -2.0, 3.0],
        },
    )
    .unwrap();
    assert_eq!(r.results.balance().origin, [1.0, -2.0, 3.0]);
    assert_eq!(
        std::fs::read(out.join("report.json")).unwrap(),
        std::fs::read(api.join("report.json")).unwrap()
    );
}
#[test]
fn missing_and_invalid_inputs_return_code_three_without_output() {
    let f = Fixture::new();
    let output = f.root.join("absent-output");
    let missing = f.root.join("missing.json");
    failed(
        &cli(
            &f.root,
            &[
                "solve".as_ref(),
                missing.as_os_str(),
                "--output".as_ref(),
                output.as_os_str(),
            ],
        ),
        3,
    );
    assert!(!output.exists());
    std::fs::write(&f.model, "{ malformed").unwrap();
    failed(
        &cli(&f.root, &["validate".as_ref(), f.model.as_os_str()]),
        3,
    );
}
#[test]
fn semantic_input_diagnostics_name_the_affected_field() {
    let f = Fixture::new();
    let text = std::fs::read_to_string(&f.model)
        .unwrap()
        .replace("1000.0", "-1.0");
    std::fs::write(&f.model, text).unwrap();
    let o = cli(&f.root, &["validate".as_ref(), f.model.as_os_str()]);
    failed(&o, 3);
    assert!(String::from_utf8_lossy(&o.stderr).contains("/materials/0/young"));
}
#[test]
fn validation_does_not_promise_convergence_and_solver_failure_returns_four() {
    let f = Fixture::new();
    f.change(|m| m.solver_settings.max_iterations = 1);
    assert!(
        cli(&f.root, &["validate".as_ref(), f.model.as_os_str()])
            .status
            .success()
    );
    let out = f.root.join("no result");
    let o = cli(
        &f.root,
        &[
            "solve".as_ref(),
            f.model.as_os_str(),
            "--output".as_ref(),
            out.as_os_str(),
        ],
    );
    failed(&o, 4);
    assert!(String::from_utf8_lossy(&o.stderr).contains("IterationBudget"));
    assert!(!out.exists());
}
#[test]
fn existing_outputs_and_missing_parents_return_five_without_overwrite() {
    let f = Fixture::new();
    let out = f.root.join("existing");
    std::fs::create_dir(&out).unwrap();
    std::fs::write(out.join("sentinel"), "keep").unwrap();
    failed(
        &cli(
            &f.root,
            &[
                "solve".as_ref(),
                f.model.as_os_str(),
                "--output".as_ref(),
                out.as_os_str(),
            ],
        ),
        5,
    );
    assert_eq!(
        std::fs::read_to_string(out.join("sentinel")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 1);
    let absent = f.root.join("absent/child");
    failed(
        &cli(
            &f.root,
            &[
                "solve".as_ref(),
                f.model.as_os_str(),
                "--output".as_ref(),
                absent.as_os_str(),
            ],
        ),
        5,
    );
    assert!(!absent.exists());
}
#[test]
fn dash_prefixed_model_and_relative_output_resolve_against_cwd() {
    let f = Fixture::new();
    std::fs::rename(&f.model, f.root.join("-model.json")).unwrap();
    let o = cli(
        &f.root,
        &[
            "solve".as_ref(),
            "--output".as_ref(),
            "relative output".as_ref(),
            "--".as_ref(),
            "-model.json".as_ref(),
        ],
    );
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(f.root.join("relative output/manifest.json").is_file());
}
#[test]
fn nodal_reference_loads_and_zero_rhs_are_solved_from_model() {
    let f = Fixture::new();
    f.change(|m| {
        m.loads = vec![Load::NodalForce {
            node: NodeId(400),
            components: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        }]
    });
    let out = f.root.join("nodal");
    assert!(
        cli(
            &f.root,
            &[
                "solve".as_ref(),
                f.model.as_os_str(),
                "--output".as_ref(),
                out.as_os_str()
            ]
        )
        .status
        .success()
    );
    f.change(|m| m.loads.clear());
    let zero = run_simulation_file(&f.model, f.root.join("zero"), &RunOptions::default()).unwrap();
    assert_eq!(zero.results.solver().iterations, 0);
    assert_eq!(zero.results.balance().scales.force(), 1.0);
}
#[cfg(target_os = "linux")]
#[test]
fn native_non_utf8_paths_are_not_lost_by_argument_parsing() {
    use std::os::unix::ffi::OsStringExt;
    let f = Fixture::new();
    let name = std::ffi::OsString::from_vec(b"model-\xff.json".to_vec());
    let path = f.root.join(name);
    std::fs::rename(&f.model, &path).unwrap();
    assert!(
        cli(&f.root, &["validate".as_ref(), path.as_os_str()])
            .status
            .success()
    );
}
