//! Model selections must be checked against actual mesh memberships.

use ferrasim_io::{
    MeshIoError, MeshedModelError, load_meshed_model, parse_model_json, parse_msh22,
};
use ferrasim_mesh::{BindingCode, ModelMeshError};
use ferrasim_model::{
    BoundaryCondition, GroupId, Load, MaterialAssignment, NodeId, PrescribedComponents, Vector3,
};

const MODEL: &str = include_str!("../../../examples/minimal/model.json");
const ONE: &str = include_str!("../../../validation/fixtures/tet4-positive.msh");
const TWO: &str = include_str!("../../../validation/fixtures/two-tets-interface.msh");

fn binding_error(
    mesh: &ferrasim_mesh::ValidatedMesh,
    model: &ferrasim_model::Simulation,
    code: BindingCode,
) {
    match mesh.validate_model(model).unwrap_err() {
        ModelMeshError::Binding(errors) => assert!(
            errors.issues.iter().any(|issue| issue.code == code),
            "{errors:?}"
        ),
        other => panic!("expected binding failure, got {other:?}"),
    }
}

#[test]
fn complete_reference_model_loads_through_portable_file_api() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/minimal/model.json");
    let loaded = load_meshed_model(path).unwrap();
    assert_eq!(loaded.mesh.mesh().nodes.len(), 4);
    assert_eq!(loaded.mesh.mesh().tetrahedra.len(), 1);
}

#[test]
fn missing_mesh_has_separate_file_error() {
    let path =
        std::env::temp_dir().join(format!("ferrasim missing mesh {}.json", std::process::id()));
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.path = format!("no-such-mesh-{}.msh", std::process::id());
    std::fs::write(&path, ferrasim_io::model_to_json(&model).unwrap()).unwrap();
    assert!(matches!(
        load_meshed_model(&path),
        Err(MeshedModelError::Mesh(MeshIoError::Read { .. }))
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn expected_groups_and_loaded_node_ids_are_checked() {
    let mesh = parse_msh22(ONE).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.nodes.push(GroupId("missing".into()));
    binding_error(&mesh, &model, BindingCode::UnknownGroup);
    let mut model = parse_model_json(MODEL).unwrap();
    model.loads[0] = Load::NodalForce {
        node: NodeId(99),
        components: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    };
    binding_error(&mesh, &model, BindingCode::UnknownNode);
}

#[test]
fn used_empty_groups_fail_but_unused_empty_surfaces_remain_visible() {
    let text = ONE
        .replace("$PhysicalNames\n2", "$PhysicalNames\n3")
        .replace("$EndPhysicalNames", "2 7 \"empty\"\n$EndPhysicalNames");
    let mesh = parse_msh22(&text).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.surfaces.push(GroupId("empty".into()));
    mesh.validate_model(&model).unwrap();
    model.loads.push(Load::SurfaceTraction {
        surface_group: GroupId("empty".into()),
        components: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    });
    binding_error(&mesh, &model, BindingCode::EmptyGroup);
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.nodes.push(GroupId("empty".into()));
    model.boundary_conditions[0].node_group = GroupId("empty".into());
    binding_error(&mesh, &model, BindingCode::EmptyGroup);
}

#[test]
fn every_tetrahedron_needs_a_named_material_region() {
    let mesh = parse_msh22(&ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 2 0 1 1 2 3 4")).unwrap();
    let model = parse_model_json(MODEL).unwrap();
    binding_error(&mesh, &model, BindingCode::UnassignedElement);
    binding_error(&mesh, &model, BindingCode::EmptyGroup);
}

#[test]
fn same_node_group_name_in_two_dimensions_is_never_silently_expanded() {
    let mesh = parse_msh22(&ONE.replace("3 2 \"solid\"", "3 2 \"base\"")).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.volumes[0] = GroupId("base".into());
    model.material_assignments[0].region = GroupId("base".into());
    binding_error(&mesh, &model, BindingCode::AmbiguousGroup);
}

#[test]
fn internal_interfaces_can_constrain_nodes_but_cannot_receive_boundary_traction() {
    let mesh = parse_msh22(TWO).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.nodes[0] = GroupId("interface".into());
    model.mesh.groups.surfaces[0] = GroupId("interface".into());
    model.boundary_conditions[0].node_group = GroupId("interface".into());
    mesh.validate_model(&model).unwrap();
    model.loads.push(Load::SurfaceTraction {
        surface_group: GroupId("interface".into()),
        components: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    });
    binding_error(&mesh, &model, BindingCode::InteriorSurface);
}

#[test]
fn overlapping_node_groups_require_consistent_component_values() {
    let mesh = parse_msh22(TWO).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.nodes = vec![GroupId("side a".into()), GroupId("side b".into())];
    model.mesh.groups.surfaces = vec![GroupId("side a".into()), GroupId("side b".into())];
    model.boundary_conditions = vec![
        BoundaryCondition {
            node_group: GroupId("side a".into()),
            components: PrescribedComponents {
                x: Some(0.0),
                y: None,
                z: None,
            },
        },
        BoundaryCondition {
            node_group: GroupId("side b".into()),
            components: PrescribedComponents {
                x: Some(0.0),
                y: None,
                z: None,
            },
        },
    ];
    mesh.validate_model(&model).unwrap();
    model.boundary_conditions[1].components.x = Some(0.001);
    binding_error(&mesh, &model, BindingCode::ConflictingPrescription);
}

#[test]
fn empty_material_regions_and_invalid_model_contracts_are_rejected() {
    let text = ONE
        .replace("$PhysicalNames\n2", "$PhysicalNames\n3")
        .replace("$EndPhysicalNames", "3 9 \"unused\"\n$EndPhysicalNames");
    let mesh = parse_msh22(&text).unwrap();
    let mut model = parse_model_json(MODEL).unwrap();
    model.mesh.groups.volumes.push(GroupId("unused".into()));
    model.material_assignments.push(MaterialAssignment {
        region: GroupId("unused".into()),
        material: model.material_assignments[0].material.clone(),
    });
    binding_error(&mesh, &model, BindingCode::EmptyGroup);
    model.schema_version = "future".into();
    assert!(matches!(
        mesh.validate_model(&model),
        Err(ModelMeshError::Model(_))
    ));
}
