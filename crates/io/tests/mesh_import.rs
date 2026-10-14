//! Independent geometry fixtures and malformed-input regression cases for S02.

use ferrasim_io::{MshCode, MshError, parse_msh22, parse_msh22_bytes};
use ferrasim_mesh::{ElementId, MeshCode};
use ferrasim_model::{GroupId, NodeId};

const ONE: &str = include_str!("../../../validation/fixtures/tet4-positive.msh");
const TWO: &str = include_str!("../../../validation/fixtures/two-tets-interface.msh");

fn format_error(text: &str, expected: MshCode) {
    match parse_msh22(text).unwrap_err() {
        MshError::Format { code, line, .. } => {
            assert_eq!(code, expected);
            assert!(line > 0);
        }
        other => panic!("expected format failure, got {other:?}"),
    }
}
fn geometry_error(text: &str, expected: MeshCode) {
    match parse_msh22(text).unwrap_err() {
        MshError::Mesh(errors) => assert!(
            errors.issues.iter().any(|issue| issue.code == expected),
            "{errors:?}"
        ),
        other => panic!("expected geometry failure, got {other:?}"),
    }
}
fn scale_coordinates(text: &str, scale: f64) -> String {
    let mut nodes = false;
    text.lines()
        .map(|line| {
            if line == "$Nodes" {
                nodes = true;
                return line.to_owned();
            }
            if line == "$EndNodes" {
                nodes = false;
                return line.to_owned();
            }
            let fields: Vec<_> = line.split_whitespace().collect();
            if nodes && fields.len() == 4 {
                format!(
                    "{} {} {} {}",
                    fields[0],
                    fields[1].parse::<f64>().unwrap() * scale,
                    fields[2].parse::<f64>().unwrap() * scale,
                    fields[3].parse::<f64>().unwrap() * scale
                )
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn known_tetrahedron_has_exact_reference_volume_surface_and_normal() {
    let mesh = parse_msh22(ONE).unwrap();
    assert_eq!(mesh.mesh().nodes.len(), 4);
    assert_eq!(mesh.geometry().len(), 1);
    assert!((mesh.geometry()[&ElementId(2)].volume - 1.0 / 6.0).abs() < 1e-14);
    let face = &mesh.boundary_faces()[&ElementId(1)];
    assert_eq!(face.owner, ElementId(2));
    assert_eq!(face.normal, [0.0, 0.0, -1.0]);
    assert!((face.area - 0.5).abs() < 1e-14);
    assert!(face.source_is_outward);
    assert_eq!(mesh.mesh().tetrahedra[&ElementId(2)].physical_tag, Some(2));
    assert_eq!(mesh.mesh().tetrahedra[&ElementId(2)].entity_tag, Some(1));
    assert_eq!(
        mesh.node_group_nodes(&GroupId("base".into())).unwrap(),
        vec![NodeId(1), NodeId(2), NodeId(3)]
    );
    assert_eq!(
        mesh.node_group_nodes(&GroupId("solid".into()))
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn inward_source_triangle_is_preserved_with_explicit_outward_metadata() {
    let text = ONE.replace("1 2 2 1 1 1 3 2", "1 2 2 1 1 1 2 3");
    let mesh = parse_msh22(&text).unwrap();
    assert_eq!(
        mesh.mesh().triangles[&ElementId(1)].nodes,
        [NodeId(1), NodeId(2), NodeId(3)]
    );
    assert!(!mesh.boundary_faces()[&ElementId(1)].source_is_outward);
    assert_eq!(
        mesh.boundary_faces()[&ElementId(1)].normal,
        [0.0, 0.0, -1.0]
    );
}

#[test]
fn shared_interface_is_valid_but_not_an_exterior_triangle() {
    let mesh = parse_msh22(TWO).unwrap();
    assert_eq!(mesh.geometry().len(), 2);
    assert!(!mesh.boundary_faces().contains_key(&ElementId(11)));
    assert_eq!(mesh.boundary_faces().len(), 2);
    assert!((mesh.geometry().values().map(|g| g.volume).sum::<f64>() - 1.0 / 3.0).abs() < 1e-14);
    assert_eq!(
        mesh.surface_groups()[&GroupId("interface".into())],
        vec![ElementId(11)]
    );
}

#[test]
fn relative_geometry_check_is_independent_of_units() {
    for scale in [1e-50, 1.0, 1e50] {
        let mesh = parse_msh22(&scale_coordinates(ONE, scale)).unwrap();
        let relative = mesh.geometry()[&ElementId(2)].volume / (scale * scale * scale / 6.0) - 1.0;
        assert!(relative.abs() < 1e-12);
        assert!(
            (mesh.boundary_faces()[&ElementId(1)].area / (scale * scale / 2.0) - 1.0).abs() < 1e-12
        );
        let flat = scale_coordinates(&ONE.replace("4 0 0 1", "4 0 0 0.0000000000001"), scale);
        geometry_error(&flat, MeshCode::Degenerate);
    }
}

#[test]
fn reversed_and_degenerate_tetrahedra_are_rejected_without_reordering() {
    geometry_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 2 2 1 1 2 4 3"),
        MeshCode::Inverted,
    );
    geometry_error(&ONE.replace("4 0 0 1", "4 0 0 0"), MeshCode::Degenerate);
    geometry_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 2 2 1 1 2 3 3"),
        MeshCode::RepeatedNode,
    );
}

#[test]
fn nonfinite_coordinates_and_unrepresentable_volumes_fail() {
    for value in ["NaN", "inf", "-inf"] {
        geometry_error(
            &ONE.replace("4 0 0 1", &format!("4 0 0 {value}")),
            MeshCode::NumericRange,
        );
    }
    geometry_error(&scale_coordinates(ONE, 1e200), MeshCode::NumericRange);
    geometry_error(&scale_coordinates(ONE, 1e-200), MeshCode::NumericRange);
}

#[test]
fn connectivity_and_unused_nodes_are_checked() {
    geometry_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 2 2 1 1 2 3 99"),
        MeshCode::MissingNode,
    );
    geometry_error(
        &ONE.replace("1 2 2 1 1 1 3 2", "1 2 2 1 1 1 3 99"),
        MeshCode::MissingNode,
    );
    let extra = ONE
        .replace("$Nodes\n4", "$Nodes\n5")
        .replace("$EndNodes", "99 4 5 6\n$EndNodes");
    geometry_error(&extra, MeshCode::UnusedNode);
}

#[test]
fn orphan_and_duplicate_triangles_are_rejected() {
    geometry_error(
        &TWO.replace("11 2 2 1 1 1 3 2", "11 2 2 1 1 2 4 5"),
        MeshCode::OrphanSurface,
    );
    let duplicate = ONE
        .replace("$Elements\n2", "$Elements\n3")
        .replace("$EndElements", "3 2 2 1 1 1 3 2\n$EndElements");
    geometry_error(&duplicate, MeshCode::DuplicateTopology);
}

#[test]
fn duplicated_tetrahedra_and_same_sided_adjacency_fail() {
    let duplicate = ONE
        .replace("$Elements\n2", "$Elements\n3")
        .replace("$EndElements", "3 4 2 2 1 1 2 3 4\n$EndElements");
    geometry_error(&duplicate, MeshCode::DuplicateTopology);
    let overlap = TWO
        .replace("5 0 0 -1", "5 0 0 2")
        .replace("21 4 2 2 2 1 3 2 5", "21 4 2 2 2 1 2 3 5");
    geometry_error(&overlap, MeshCode::InvalidAdjacency);
}

#[test]
fn nonmanifold_face_with_three_owners_fails() {
    let text = TWO
        .replace("$Nodes\n5", "$Nodes\n6")
        .replace("$EndNodes", "6 0 0 2\n$EndNodes")
        .replace("$Elements\n5", "$Elements\n6")
        .replace("$EndElements", "22 4 2 2 3 1 2 3 6\n$EndElements");
    geometry_error(&text, MeshCode::InvalidAdjacency);
}

#[test]
fn sparse_unordered_ids_and_section_order_are_preserved() {
    let mesh = parse_msh22(ONE).unwrap();
    let mut raw = mesh.mesh().clone();
    let coordinates = raw.nodes.remove(&NodeId(4)).unwrap();
    raw.nodes.insert(NodeId(9999), coordinates);
    raw.tetrahedra.get_mut(&ElementId(2)).unwrap().nodes[3] = NodeId(9999);
    assert!(raw.into_validated().is_ok());
    let physical = ONE
        .split("$PhysicalNames")
        .nth(1)
        .unwrap()
        .split("$EndPhysicalNames")
        .next()
        .unwrap();
    let nodes = ONE
        .split("$Nodes")
        .nth(1)
        .unwrap()
        .split("$EndNodes")
        .next()
        .unwrap();
    let elements = ONE
        .split("$Elements")
        .nth(1)
        .unwrap()
        .split("$EndElements")
        .next()
        .unwrap();
    let text = format!(
        "$MeshFormat\n2.2 0 8\n$EndMeshFormat\n$Elements{elements}$EndElements\n$Nodes{nodes}$EndNodes\n$PhysicalNames{physical}$EndPhysicalNames\n"
    );
    let other = parse_msh22(&text).unwrap();
    assert_eq!(other.mesh().tetrahedra, mesh.mesh().tetrahedra);
    let sparse = ONE
        .replace("4 0 0 1", "9999 0 0 1")
        .replace("2 4 2 2 1 1 2 3 4", "5000 4 2 2 1 1 2 3 9999");
    let mesh = parse_msh22(&sparse).unwrap();
    assert!(mesh.mesh().nodes.contains_key(&NodeId(9999)));
    assert!(mesh.geometry().contains_key(&ElementId(5000)));
}

#[test]
fn binary_and_other_versions_are_rejected_before_decoding_payload() {
    for header in ["4.1 0 8", "2.2 1 8", "2.2 0 4"] {
        format_error(&ONE.replace("2.2 0 8", header), MshCode::UnsupportedFormat);
    }
    let binary = b"$MeshFormat\n2.2 1 8\n\xff\x00\x00\x00";
    assert!(matches!(
        parse_msh22_bytes(binary),
        Err(MshError::Format {
            code: MshCode::UnsupportedFormat,
            ..
        })
    ));
}

#[test]
fn format_diagnostics_keep_real_lines_even_after_blank_lines() {
    let bytes = b"\n\n$MeshFormat\n2.2 1 8\n\xff";
    assert!(matches!(
        parse_msh22_bytes(bytes),
        Err(MshError::Format {
            code: MshCode::UnsupportedFormat,
            line: 4,
            ..
        })
    ));
}

#[test]
fn empty_volumes_and_zero_ids_are_rejected() {
    let text = ONE
        .replace("$Elements\n2", "$Elements\n1")
        .replace("2 4 2 2 1 1 2 3 4\n", "");
    geometry_error(&text, MeshCode::Empty);
    geometry_error(
        &ONE.replace("1 2 2 1 1 1 3 2", "0 2 2 1 1 1 3 2"),
        MeshCode::InvalidId,
    );
    geometry_error(&ONE.replace("1 0 0 0", "0 0 0 0"), MeshCode::InvalidId);
}

#[test]
fn unsupported_elements_sections_and_partition_metadata_are_explicit_errors() {
    format_error(
        &ONE.replace("1 2 2 1 1 1 3 2", "1 3 2 1 1 1 3 2 4"),
        MshCode::UnsupportedElement,
    );
    format_error(
        &format!("{ONE}\n$Periodic\n0\n$EndPeriodic"),
        MshCode::UnsupportedSection,
    );
    format_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 3 2 1 0 1 2 3 4"),
        MshCode::UnsupportedTags,
    );
}

#[test]
fn duplicates_counts_and_truncation_never_overwrite_or_panic() {
    format_error(&ONE.replace("2 1 0 0", "1 1 0 0"), MshCode::DuplicateId);
    format_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "1 4 2 2 1 1 2 3 4"),
        MshCode::DuplicateId,
    );
    for text in [
        "",
        "$MeshFormat\n2.2 0 8",
        "$MeshFormat\n2.2 0 8\n$EndMeshFormat\n$Nodes\n18446744073709551615",
        "$MeshFormat\n2.2 0 8\n$EndMeshFormat\n$Nodes\n-1",
    ] {
        format_error(text, MshCode::Syntax);
    }
    format_error(
        &ONE.replace("2 4 2 2 1 1 2 3 4", "2 4 2 2 1 1 2 3"),
        MshCode::Syntax,
    );
    format_error(&ONE.replace("$EndNodes", "$EndElements"), MshCode::Syntax);
    let nodes = ONE
        .split("$Nodes")
        .nth(1)
        .unwrap()
        .split("$EndNodes")
        .next()
        .unwrap();
    format_error(&format!("{ONE}$Nodes{nodes}$EndNodes"), MshCode::Syntax);
}

#[test]
fn physical_names_are_unique_and_unnamed_tags_are_retained() {
    let duplicate = ONE
        .replace("$PhysicalNames\n2", "$PhysicalNames\n3")
        .replace("$EndPhysicalNames", "2 1 \"base copy\"\n$EndPhysicalNames");
    geometry_error(&duplicate, MeshCode::PhysicalGroup);
    let same_name = ONE
        .replace("$PhysicalNames\n2", "$PhysicalNames\n3")
        .replace("$EndPhysicalNames", "2 9 \"base\"\n$EndPhysicalNames");
    geometry_error(&same_name, MeshCode::PhysicalGroup);
    let begin = ONE.find("$PhysicalNames").unwrap();
    let end = ONE.find("$EndPhysicalNames").unwrap() + "$EndPhysicalNames".len();
    let without = format!("{}{}", &ONE[..begin], &ONE[end..]);
    let mesh = parse_msh22(&without).unwrap();
    assert!(mesh.volume_groups().is_empty());
    assert_eq!(mesh.mesh().tetrahedra[&ElementId(2)].physical_tag, Some(2));
}

#[test]
fn utf8_names_crlf_and_extra_whitespace_are_supported() {
    let text = ONE
        .replace("\"base\"", "\"face pièce\"")
        .replace('\n', "\r\n");
    let mesh = parse_msh22_bytes(text.as_bytes()).unwrap();
    assert_eq!(
        mesh.node_group_nodes(&GroupId("face pièce".into()))
            .unwrap()
            .len(),
        3
    );
    assert!(parse_msh22(&format!("\n\n{ONE}\n")).is_ok());
}

#[test]
fn api_mesh_validation_catches_invalid_ids_and_geometry() {
    let mesh = parse_msh22(ONE).unwrap();
    let mut raw = mesh.mesh().clone();
    raw.nodes.get_mut(&NodeId(4)).unwrap()[2] = f64::NAN;
    assert!(
        raw.into_validated()
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == MeshCode::NumericRange)
    );
    let mut raw = mesh.mesh().clone();
    raw.tetrahedra.get_mut(&ElementId(2)).unwrap().id = ElementId(99);
    assert!(
        raw.into_validated()
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == MeshCode::InvalidId)
    );
    let mut raw = mesh.mesh().clone();
    raw.tetrahedra.get_mut(&ElementId(2)).unwrap().physical_tag = Some(0);
    assert!(
        raw.into_validated()
            .unwrap_err()
            .issues
            .iter()
            .any(|issue| issue.code == MeshCode::PhysicalGroup)
    );
}

#[test]
fn every_tetrahedron_face_has_outward_geometry() {
    let text = ONE.replace("$Elements\n2", "$Elements\n5").replace(
        "$EndElements",
        "3 2 2 1 1 2 3 4\n4 2 2 1 1 1 4 3\n5 2 2 1 1 1 2 4\n$EndElements",
    );
    let mesh = parse_msh22(&text).unwrap();
    assert_eq!(mesh.boundary_faces().len(), 4);
    let centroid = [0.25, 0.25, 0.25];
    for face in mesh.boundary_faces().values() {
        let point = mesh.mesh().nodes[&face.outward_nodes[0]];
        let inward_dot: f64 = (0..3)
            .map(|i| face.normal[i] * (centroid[i] - point[i]))
            .sum();
        assert!(inward_dot < 0.0);
        assert!(face.source_is_outward);
    }
    let slanted = &mesh.boundary_faces()[&ElementId(3)];
    assert!((slanted.area - 3.0_f64.sqrt() / 2.0).abs() < 1e-14);
    for component in slanted.normal {
        assert!((component - 1.0 / 3.0_f64.sqrt()).abs() < 1e-14);
    }
}

#[test]
fn zero_and_single_tag_records_have_explicit_absent_metadata() {
    let text = ONE
        .replace("1 2 2 1 1 1 3 2", "1 2 0 1 3 2")
        .replace("2 4 2 2 1 1 2 3 4", "2 4 1 2 1 2 3 4");
    let mesh = parse_msh22(&text).unwrap();
    assert_eq!(mesh.mesh().triangles[&ElementId(1)].physical_tag, None);
    assert_eq!(mesh.mesh().tetrahedra[&ElementId(2)].physical_tag, Some(2));
    assert_eq!(mesh.mesh().tetrahedra[&ElementId(2)].entity_tag, None);
}
