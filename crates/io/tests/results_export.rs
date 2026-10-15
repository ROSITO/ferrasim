//! Result snapshot, format identity, exact external IDs and conservative output handling.
use ferrasim_fem::{BalanceScales, SimulationResults, solve_static_results};
use ferrasim_io::{
    ResultProvenance, export_results, load_meshed_model, results_to_elements_csv, results_to_json,
    results_to_nodes_csv, results_to_vtu,
};
use ferrasim_model::{CoherentUnits, Material, MaterialId, NodeId, OutputFormat, Units};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn input() -> ferrasim_io::MeshedSimulation {
    load_meshed_model(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/uniaxial/model.json"),
    )
    .unwrap()
}
fn solve(loaded: &ferrasim_io::MeshedSimulation) -> SimulationResults {
    solve_static_results(
        &loaded.mesh,
        &loaded.model.simulation,
        [0.0; 3],
        BalanceScales::new(10.0, 2.0).unwrap(),
    )
    .unwrap()
}
fn source() -> ResultProvenance {
    ResultProvenance {
        model_source: "modèle, \"test\".json".into(),
        mesh_source: "mesh.msh".into(),
        declared_code_revision: None,
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ferrasim S07 é {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[test]
fn json_report_preserves_api_fields_units_snapshots_and_solver_diagnostics() {
    let r = solve(&input());
    let json: Value = serde_json::from_str(&results_to_json(&r, &source()).unwrap()).unwrap();
    assert_eq!(json["result_version"], 1);
    assert_eq!(json["analysis_status"], "converged");
    assert_eq!(
        json["scientific_qualification"],
        "pending_independent_review"
    );
    assert_eq!(json["provenance"]["model_source"], source().model_source);
    assert!(json["provenance"]["declared_code_revision"].is_null());
    assert_eq!(
        json["provenance"]["model_snapshot"],
        serde_json::to_value(r.model()).unwrap()
    );
    assert_eq!(
        json["provenance"]["mesh_snapshot"]["triangles"]
            .as_array()
            .unwrap()
            .len(),
        12
    );
    assert_eq!(
        json["solver"]["iterations"].as_u64().unwrap(),
        r.solver().iterations
    );
    assert_eq!(
        json["balance"]["strain_energy"].as_f64().unwrap(),
        r.balance().strain_energy
    );
    for (row, n) in json["nodes"].as_array().unwrap().iter().zip(r.nodes()) {
        assert_eq!(row["id"].as_u64().unwrap(), n.id.0);
        for i in 0..3 {
            assert_eq!(row["reaction"][i].as_f64(), n.reaction[i]);
        }
    }
    for (row, e) in json["elements"]
        .as_array()
        .unwrap()
        .iter()
        .zip(r.elements())
    {
        assert_eq!(row["von_mises"].as_f64().unwrap(), e.von_mises);
        for i in 0..6 {
            assert_eq!(row["stress"][i].as_f64().unwrap(), e.stress[i]);
        }
    }
}
#[test]
fn custom_units_and_quoted_material_labels_survive_csv_and_json() {
    let mut loaded = input();
    let id = MaterialId("matériau, \"A<&>\"".into());
    loaded.model.simulation.materials = vec![Material::IsotropicLinearElastic {
        id: id.clone(),
        young: 1000.0,
        poisson: 0.25,
    }];
    loaded.model.simulation.material_assignments[0].material = id.clone();
    loaded.model.simulation.units = Units::Coherent(CoherentUnits {
        length: "mm".into(),
        force: "N".into(),
    });
    let r = solve(&loaded);
    let csv = results_to_elements_csv(&r);
    assert!(csv.contains("\"matériau, \"\"A<&>\"\"\""));
    let json: Value = serde_json::from_str(&results_to_json(&r, &source()).unwrap()).unwrap();
    assert_eq!(json["units"]["length"], "mm");
    assert_eq!(json["elements"][0]["material"], id.0);
    let vtu = results_to_vtu(&r).unwrap();
    assert!(!vtu.contains("matériau"));
    assert!(vtu.contains("MaterialIndex"));
}
#[test]
fn vtu_keeps_uint64_ids_above_float_integer_precision_and_zero_based_connectivity() {
    let mut loaded = input();
    let mut raw = loaded.mesh.mesh().clone();
    let base = 9_007_199_254_740_993_u64;
    raw.nodes = raw
        .nodes
        .into_iter()
        .map(|(id, x)| (NodeId(id.0 + base), x))
        .collect();
    for t in raw.tetrahedra.values_mut() {
        for node in &mut t.nodes {
            node.0 += base;
        }
    }
    for t in raw.triangles.values_mut() {
        for node in &mut t.nodes {
            node.0 += base;
        }
    }
    loaded.mesh = raw.into_validated().unwrap();
    let r = solve(&loaded);
    let xml = results_to_vtu(&r).unwrap();
    let json: Value = serde_json::from_str(&results_to_json(&r, &source()).unwrap()).unwrap();
    assert!(xml.contains("type=\"UInt64\" Name=\"NodeId\""));
    assert!(xml.contains(&r.nodes()[0].id.0.to_string()));
    assert_eq!(json["nodes"][0]["id"].as_u64().unwrap(), r.nodes()[0].id.0);
    assert!(xml.contains("type=\"Int64\" Name=\"connectivity\""));
    let p = xml.split("<CellData").next().unwrap();
    assert!(!p.contains("StressVoigt"));
    assert!(xml.contains("NumberOfComponents=\"6\""));
}
#[test]
fn new_unicode_directory_has_all_selected_files_and_manifest_written_last() {
    let temp = Temp::new();
    let path = temp.0.join("résultats complets");
    let r = solve(&input());
    let exported = export_results(&r, &source(), &path).unwrap();
    assert_eq!(exported.files.last().unwrap(), "manifest.json");
    assert_eq!(
        std::fs::read_to_string(path.join("nodes.csv")).unwrap(),
        results_to_nodes_csv(&r)
    );
    assert_eq!(
        std::fs::read_to_string(path.join("elements.csv")).unwrap(),
        results_to_elements_csv(&r)
    );
    assert_eq!(
        std::fs::read_to_string(path.join("results.vtu")).unwrap(),
        results_to_vtu(&r).unwrap()
    );
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(path.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["status"], "complete");
    for file in manifest["files"].as_array().unwrap() {
        assert_eq!(
            std::fs::metadata(path.join(file["path"].as_str().unwrap()))
                .unwrap()
                .len(),
            file["bytes"].as_u64().unwrap()
        );
    }
    assert!(!path.join(".manifest.tmp").exists());
}
#[test]
fn existing_destinations_and_missing_parents_fail_without_overwrite() {
    let temp = Temp::new();
    let r = solve(&input());
    let target = temp.0.join("existing");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("sentinel"), "keep").unwrap();
    assert!(export_results(&r, &source(), &target).is_err());
    assert_eq!(
        std::fs::read_to_string(target.join("sentinel")).unwrap(),
        "keep"
    );
    assert!(!target.join("manifest.json").exists());
    let target = temp.0.join("absent/child");
    assert!(export_results(&r, &source(), &target).is_err());
    assert!(!target.exists());
}
#[test]
fn requested_formats_are_honored_without_creating_unrequested_field_files() {
    let mut loaded = input();
    loaded.model.simulation.outputs.formats = vec![OutputFormat::Json];
    let r = solve(&loaded);
    let temp = Temp::new();
    let path = temp.0.join("json-only");
    let out = export_results(&r, &source(), &path).unwrap();
    assert_eq!(out.files, vec!["report.json", "manifest.json"]);
    assert!(!path.join("results.vtu").exists());
}
