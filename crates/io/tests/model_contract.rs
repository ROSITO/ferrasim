//! Behavioral checks for the S01 contract, including invalid user inputs.

use ferrasim_io::{ModelIoError, load_model, model_to_json, parse_model_json};
use ferrasim_model::{IssueCode, Load, Material, Units};
use serde_json::{Value, json};

const EXAMPLE: &str = include_str!("../../../examples/minimal/model.json");

fn fixture() -> Value {
    serde_json::from_str(EXAMPLE).unwrap()
}

fn parse(value: &Value) -> Result<ferrasim_model::Simulation, ModelIoError> {
    parse_model_json(&serde_json::to_string(value).unwrap())
}

fn semantic_issue(value: &Value, code: IssueCode, path: &str) {
    match parse(value).unwrap_err() {
        ModelIoError::Validation(errors) => assert!(
            errors
                .issues
                .iter()
                .any(|issue| issue.code == code && issue.path == path),
            "expected {code:?} at {path}, found {errors:?}"
        ),
        other => panic!("expected semantic error, found {other:?}"),
    }
}

#[test]
fn all_supported_loads_and_prescriptions_round_trip() {
    let mut value = fixture();
    value["boundary_conditions"][0]["components"] = json!({"x": 0.002, "z": 0.0});
    value["loads"].as_array_mut().unwrap().push(json!({
        "type": "surface_traction", "surface_group": "base",
        "components": {"x": 2.0, "y": -3.0, "z": 0.0}
    }));
    let model = parse(&value).unwrap();
    assert_eq!(model.boundary_conditions[0].components.y, None);
    assert_eq!(model.boundary_conditions[0].components.x, Some(0.002));
    assert!(matches!(model.loads[1], Load::SurfaceTraction { .. }));
    assert_eq!(
        parse_model_json(&model_to_json(&model).unwrap()).unwrap(),
        model
    );
}

#[test]
fn coherent_units_are_explicit_and_preserved() {
    let mut value = fixture();
    value["units"] = json!({"length": "mm", "force": "N"});
    let model = parse(&value).unwrap();
    assert!(matches!(model.units, Units::Coherent(_)));
    assert_eq!(
        parse_model_json(&model_to_json(&model).unwrap()).unwrap(),
        model
    );
}

#[test]
fn unknown_version_is_rejected_before_current_shape() {
    assert!(matches!(
        parse_model_json(r#"{"schema_version":"9.0","future_field":true}"#),
        Err(ModelIoError::UnsupportedVersion { found }) if found == "9.0"
    ));
}

#[test]
fn missing_and_unknown_fields_never_supply_silent_defaults() {
    for pointer in [
        "",
        "/mesh",
        "/materials/0",
        "/solver_settings",
        "/analysis",
        "/loads/0",
        "/boundary_conditions/0/components",
    ] {
        let mut value = fixture();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("typo_or_future_field".into(), json!(42));
        assert!(
            matches!(parse(&value), Err(ModelIoError::Json(_))),
            "{pointer}"
        );
    }
    for key in [
        "materials",
        "loads",
        "analysis",
        "solver_settings",
        "outputs",
    ] {
        let mut value = fixture();
        value.as_object_mut().unwrap().remove(key);
        assert!(matches!(parse(&value), Err(ModelIoError::Json(_))), "{key}");
    }
}

#[test]
fn repeated_json_fields_are_rejected() {
    let duplicate = EXAMPLE.replace(
        "\"young\": 210000000000.0",
        "\"young\": 210000000000.0, \"young\": 1.0",
    );
    assert_ne!(duplicate, EXAMPLE);
    assert!(matches!(
        parse_model_json(&duplicate),
        Err(ModelIoError::Json(_))
    ));
    let duplicate = EXAMPLE.replace(
        "\"schema_version\": \"0.1\"",
        "\"schema_version\": \"0.1\", \"schema_version\": \"0.1\"",
    );
    assert!(matches!(
        parse_model_json(&duplicate),
        Err(ModelIoError::Json(_))
    ));
}

#[test]
fn incompatible_types_and_unsupported_procedures_are_rejected() {
    for (pointer, replacement) in [
        ("/analysis/type", json!("nonlinear_static")),
        ("/materials/0/type", json!("plastic")),
        ("/solver_settings/method", json!("gmres")),
        ("/solver_settings/preconditioner", json!("ilu")),
        ("/outputs/formats/0", json!("hdf5")),
        ("/loads/0/node", json!(-1)),
        ("/loads/0/node", json!(1.5)),
        ("/materials/0/young", Value::Null),
    ] {
        let mut value = fixture();
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            matches!(parse(&value), Err(ModelIoError::Json(_))),
            "{pointer}"
        );
    }
    assert!(matches!(parse_model_json("{"), Err(ModelIoError::Json(_))));
}

#[test]
fn materials_must_have_valid_constants_and_unique_ids() {
    for (field, invalid) in [
        ("young", 0.0),
        ("young", -1.0),
        ("poisson", -1.0),
        ("poisson", 0.5),
    ] {
        let mut value = fixture();
        value["materials"][0][field] = json!(invalid);
        semantic_issue(
            &value,
            IssueCode::OutOfRange,
            &format!("/materials/0/{field}"),
        );
    }
    let mut value = fixture();
    let duplicate = value["materials"][0].clone();
    value["materials"].as_array_mut().unwrap().push(duplicate);
    semantic_issue(&value, IssueCode::Duplicate, "/materials/1/id");
}

#[test]
fn all_declared_references_are_checked() {
    for (pointer, path) in [
        (
            "/material_assignments/0/material",
            "/material_assignments/0/material",
        ),
        (
            "/material_assignments/0/region",
            "/material_assignments/0/region",
        ),
        (
            "/boundary_conditions/0/node_group",
            "/boundary_conditions/0/node_group",
        ),
    ] {
        let mut value = fixture();
        *value.pointer_mut(pointer).unwrap() = json!("missing");
        semantic_issue(&value, IssueCode::UnknownReference, path);
    }
    let mut value = fixture();
    value["loads"][0] = json!({"type":"surface_traction", "surface_group":"missing", "components":{"x":0.0,"y":0.0,"z":1.0}});
    semantic_issue(
        &value,
        IssueCode::UnknownReference,
        "/loads/0/surface_group",
    );
}

#[test]
fn volumes_require_exactly_one_assignment() {
    let mut value = fixture();
    value["material_assignments"] = json!([]);
    semantic_issue(
        &value,
        IssueCode::MissingAssignment,
        "/material_assignments",
    );
    let mut value = fixture();
    let duplicate = value["material_assignments"][0].clone();
    value["material_assignments"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    semantic_issue(
        &value,
        IssueCode::Duplicate,
        "/material_assignments/1/region",
    );
}

#[test]
fn group_names_are_unique_within_categories_and_nonempty() {
    let mut value = fixture();
    value["mesh"]["groups"]["nodes"] = json!(["base", "base"]);
    semantic_issue(&value, IssueCode::Duplicate, "/mesh/groups/nodes/1");
    for name in ["", " base", "base\n"] {
        let mut value = fixture();
        value["mesh"]["groups"]["nodes"][0] = json!(name);
        semantic_issue(&value, IssueCode::EmptyValue, "/mesh/groups/nodes/0");
    }
}

#[test]
fn conflicting_and_empty_prescriptions_fail_but_consistent_repeats_pass() {
    let mut value = fixture();
    let identical = value["boundary_conditions"][0].clone();
    value["boundary_conditions"]
        .as_array_mut()
        .unwrap()
        .push(identical);
    parse(&value).unwrap();
    value["boundary_conditions"][1]["components"]["x"] = json!(0.001);
    semantic_issue(
        &value,
        IssueCode::InvalidPrescription,
        "/boundary_conditions/1/components/x",
    );
    let mut value = fixture();
    value["boundary_conditions"][0]["components"] = json!({});
    semantic_issue(
        &value,
        IssueCode::InvalidPrescription,
        "/boundary_conditions/0/components",
    );
}

#[test]
fn numerical_settings_and_output_requests_are_checked() {
    for (pointer, replacement, path) in [
        (
            "/solver_settings/rtol",
            json!(-1.0),
            "/solver_settings/rtol",
        ),
        (
            "/solver_settings/atol",
            json!(-1.0),
            "/solver_settings/atol",
        ),
        (
            "/solver_settings/max_iterations",
            json!(0),
            "/solver_settings/max_iterations",
        ),
        ("/loads/0/node", json!(0), "/loads/0/node"),
    ] {
        let mut value = fixture();
        *value.pointer_mut(pointer).unwrap() = replacement;
        semantic_issue(&value, IssueCode::OutOfRange, path);
    }
    let mut value = fixture();
    value["solver_settings"]["rtol"] = json!(0.0);
    value["solver_settings"]["atol"] = json!(0.0);
    semantic_issue(&value, IssueCode::OutOfRange, "/solver_settings");
    value["solver_settings"]["atol"] = json!(1e-12);
    parse(&value).unwrap();
    let mut value = fixture();
    value["outputs"]["formats"] = json!(["vtu", "vtu"]);
    semantic_issue(&value, IssueCode::Duplicate, "/outputs/formats/1");
    value["outputs"]["formats"] = json!([]);
    semantic_issue(&value, IssueCode::EmptyValue, "/outputs/formats");
}

#[test]
fn api_nonfinite_values_are_rejected_before_serialization() {
    let mut model = parse_model_json(EXAMPLE).unwrap();
    let Material::IsotropicLinearElastic { young, .. } = &mut model.materials[0];
    *young = f64::NAN;
    model.boundary_conditions[0].components.z = Some(f64::INFINITY);
    model.solver_settings.rtol = f64::NEG_INFINITY;
    let errors = model.validate().unwrap_err();
    assert_eq!(
        errors
            .issues
            .iter()
            .filter(|issue| issue.code == IssueCode::NonFinite)
            .count(),
        3
    );
    assert!(matches!(
        model_to_json(&model),
        Err(ModelIoError::Validation(_))
    ));
}

#[test]
fn portable_mesh_paths_allow_unicode_and_spaces_but_reject_absolute_paths() {
    for path in ["maillages/pièce acier.msh", "../mesh.msh"] {
        let mut value = fixture();
        value["mesh"]["path"] = json!(path);
        parse(&value).unwrap();
    }
    for path in [
        "",
        ".",
        "..",
        "mesh/..",
        "/tmp/mesh.msh",
        "C:/mesh.msh",
        r"mesh\part.msh",
        "mesh/",
        " mesh.msh",
    ] {
        let mut value = fixture();
        value["mesh"]["path"] = json!(path);
        semantic_issue(&value, IssueCode::InvalidPath, "/mesh/path");
    }
}

#[test]
fn loading_resolves_relative_to_unicode_source_without_needing_mesh() {
    let unique = format!(
        "ferrasim S01 pièce-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let directory = std::env::temp_dir().join(unique);
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("modèle.json");
    let mut value = fixture();
    value["mesh"]["path"] = json!("maillages/pièce.msh");
    let crlf = serde_json::to_string_pretty(&value)
        .unwrap()
        .replace('\n', "\r\n");
    std::fs::write(&source, crlf).unwrap();
    let loaded = load_model(&source).unwrap();
    assert_eq!(loaded.source_path, source.canonicalize().unwrap());
    assert_eq!(
        loaded.mesh_path,
        directory
            .canonicalize()
            .unwrap()
            .join("maillages")
            .join("pièce.msh")
    );
    assert!(!loaded.mesh_path.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn missing_model_returns_read_error_with_source() {
    let source =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/absent-model.json");
    assert!(matches!(load_model(&source), Err(ModelIoError::Read { path, .. }) if path == source));
}

#[test]
fn issues_are_aggregated_deterministically_for_api_consumers() {
    let mut model = parse_model_json(EXAMPLE).unwrap();
    model.schema_version = "future".into();
    model.material_assignments.clear();
    model.outputs.formats.clear();
    let first = model.validate().unwrap_err();
    assert_eq!(first, model.validate().unwrap_err());
    assert_eq!(first.issues.len(), 3);
    assert_eq!(first.issues[0].code, IssueCode::UnsupportedVersion);
    assert_eq!(first.issues[1].code, IssueCode::MissingAssignment);
    assert_eq!(first.issues[2].code, IssueCode::EmptyValue);
    let encoded = serde_json::to_value(&first.issues[0]).unwrap();
    assert_eq!(encoded["code"], "unsupported_version");
}
