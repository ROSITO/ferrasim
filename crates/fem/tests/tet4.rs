//! Element-level verification against exact independent tensor references.

use ferrasim_fem::{
    ElementDisplacements, ElementStiffness, FemError, IsotropicElastic, Tet4Element, VoigtVector,
};
use nalgebra::{Matrix3, SMatrix, Vector3};
use serde::Deserialize;

#[derive(Deserialize)]
struct Reference {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    coordinates: [[f64; 3]; 4],
    young: f64,
    poisson: String,
    volume: String,
    gradients: [[String; 3]; 4],
    stiffness: [[String; 12]; 12],
}
fn cases() -> Vec<Case> {
    serde_json::from_str::<Reference>(include_str!(
        "../../../validation/fixtures/tet4-reference.json"
    ))
    .unwrap()
    .cases
}
fn fraction(text: &str) -> f64 {
    match text.split_once('/') {
        Some((a, b)) => a.parse::<f64>().unwrap() / b.parse::<f64>().unwrap(),
        None => text.parse().unwrap(),
    }
}
fn affine(
    coordinates: &[[f64; 3]; 4],
    gradient: &Matrix3<f64>,
    translation: Vector3<f64>,
) -> ElementDisplacements {
    ElementDisplacements::from_fn(|index, _| {
        let node = index / 3;
        let axis = index % 3;
        (gradient * Vector3::from(coordinates[node]) + translation)[axis]
    })
}

#[test]
fn full_stiffness_gradients_and_volume_match_exact_dense_reference() {
    for case in cases() {
        let element = Tet4Element::from_coordinates(case.coordinates).unwrap();
        let law = IsotropicElastic::new(case.young, fraction(&case.poisson)).unwrap();
        let expected = ElementStiffness::from_fn(|i, j| fraction(&case.stiffness[i][j]));
        let error = (element.stiffness(&law).unwrap() - expected).norm() / expected.norm();
        assert!(error <= 1e-12, "{}: dense error {error}", case.name);
        assert!((element.volume() / fraction(&case.volume) - 1.0).abs() < 1e-12);
        let gradients =
            SMatrix::<f64, 3, 4>::from_fn(|axis, node| fraction(&case.gradients[node][axis]));
        assert!((element.gradients() - gradients).norm() / gradients.norm() < 1e-12);
    }
}

#[test]
fn shape_functions_reproduce_nodes_partition_and_affine_position() {
    let corners = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    for (i, point) in corners.into_iter().enumerate() {
        let values = Tet4Element::shape_functions(point).unwrap();
        for (j, value) in values.into_iter().enumerate() {
            assert_eq!(value, if i == j { 1.0 } else { 0.0 });
        }
    }
    for case in cases() {
        let element = Tet4Element::from_coordinates(case.coordinates).unwrap();
        let p = [0.2, 0.3, 0.1];
        let n = Tet4Element::shape_functions(p).unwrap();
        assert!((n.iter().sum::<f64>() - 1.0).abs() < 1e-14);
        let mapped: Vector3<f64> = (0..4)
            .map(|i| Vector3::from(case.coordinates[i]) * n[i])
            .sum();
        let expected = Vector3::from(case.coordinates[0]) + element.jacobian() * Vector3::from(p);
        assert!((mapped - expected).norm() < 1e-13);
    }
    assert!(Tet4Element::shape_functions([f64::NAN, 0.0, 0.0]).is_err());
}

#[test]
fn stiffness_is_symmetric_with_exactly_six_rigid_modes() {
    for case in cases() {
        let element = Tet4Element::from_coordinates(case.coordinates).unwrap();
        let law = IsotropicElastic::new(case.young, fraction(&case.poisson)).unwrap();
        let k = element.stiffness(&law).unwrap();
        let symmetry = (k - k.transpose()).norm() / k.norm();
        assert!(symmetry <= 1e-10);
        let mut worst: f64 = 0.0;
        for axis in 0..3 {
            let mut translation = Vector3::zeros();
            translation[axis] = 1.0;
            let displacement = affine(element.coordinates(), &Matrix3::zeros(), translation);
            worst = worst.max((k * displacement).norm() / (k.norm() * displacement.norm()));
            let mut rotation = Matrix3::zeros();
            let a = (axis + 1) % 3;
            let b = (axis + 2) % 3;
            rotation[(a, b)] = -1.0;
            rotation[(b, a)] = 1.0;
            let displacement = affine(element.coordinates(), &rotation, Vector3::zeros());
            worst = worst.max((k * displacement).norm() / (k.norm() * displacement.norm()));
        }
        assert!(worst <= 1e-10, "{} rigid-mode error {worst}", case.name);
        let eigen = (k / k.norm()).symmetric_eigen().eigenvalues;
        assert_eq!(eigen.iter().filter(|value| value.abs() < 1e-10).count(), 6);
        assert_eq!(eigen.iter().filter(|value| **value > 1e-10).count(), 6);
        println!(
            "{}: symmetry={symmetry:.4e}, rigid_modes={worst:.4e}",
            case.name
        );
    }
}

#[test]
fn arbitrary_affine_field_matches_tensor_strain_stress_and_energy() {
    let a = Matrix3::from_row_slice(&[
        0.002, -0.003, 0.004, 0.005, -0.001, 0.006, -0.007, 0.008, 0.003,
    ]);
    let epsilon = (a + a.transpose()) * 0.5;
    let expected_strain = VoigtVector::from_row_slice(&[
        epsilon[(0, 0)],
        epsilon[(1, 1)],
        epsilon[(2, 2)],
        2.0 * epsilon[(0, 1)],
        2.0 * epsilon[(1, 2)],
        2.0 * epsilon[(0, 2)],
    ]);
    for case in cases() {
        let element = Tet4Element::from_coordinates(case.coordinates).unwrap();
        let poisson = fraction(&case.poisson);
        let young = case.young;
        let law = IsotropicElastic::new(young, poisson).unwrap();
        let u = affine(element.coordinates(), &a, Vector3::new(0.13, -0.27, 0.41));
        let response = element.response(&law, &u).unwrap();
        assert!((response.strain - expected_strain).norm() / expected_strain.norm() < 1e-8);
        let mu = young / (2.0 * (1.0 + poisson));
        let lambda = young * poisson / ((1.0 + poisson) * (1.0 - 2.0 * poisson));
        let sigma = Matrix3::identity() * (lambda * epsilon.trace()) + epsilon * (2.0 * mu);
        let expected = VoigtVector::from_row_slice(&[
            sigma[(0, 0)],
            sigma[(1, 1)],
            sigma[(2, 2)],
            sigma[(0, 1)],
            sigma[(1, 2)],
            sigma[(0, 2)],
        ]);
        assert!((response.stress - expected).norm() / expected.norm() < 1e-8);
        let continuum = (lambda / 2.0 * epsilon.trace().powi(2) + mu * epsilon.norm_squared())
            * element.volume();
        let discrete = 0.5 * u.dot(&(element.stiffness(&law).unwrap() * u));
        assert!((response.energy / continuum - 1.0).abs() < 1e-10);
        assert!((discrete / continuum - 1.0).abs() < 1e-10);
    }
}

#[test]
fn stiffness_scales_with_length_and_youngs_modulus() {
    let coordinates = cases().remove(1).coordinates;
    let base = Tet4Element::from_coordinates(coordinates).unwrap();
    let law = IsotropicElastic::new(150.0, 0.25).unwrap();
    let k = base.stiffness(&law).unwrap();
    for scale in [1e-50, 1e-3, 1e3, 1e50] {
        let scaled =
            Tet4Element::from_coordinates(coordinates.map(|p| p.map(|v| v * scale))).unwrap();
        assert!((scaled.stiffness(&law).unwrap() / scale - k).norm() / k.norm() < 1e-12);
    }
    let stiff = base
        .stiffness(&IsotropicElastic::new(1500.0, 0.25).unwrap())
        .unwrap();
    assert!((stiff / 10.0 - k).norm() / k.norm() < 1e-12);
}

#[test]
fn isotropic_stiffness_transforms_consistently_under_rigid_frame_change() {
    let coordinates = cases().remove(1).coordinates;
    let q = Matrix3::from_row_slice(&[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    let moved = coordinates.map(|p| (q * Vector3::from(p) + Vector3::new(4.0, -2.0, 3.0)).into());
    let law = IsotropicElastic::new(150.0, 0.25).unwrap();
    let original = Tet4Element::from_coordinates(coordinates)
        .unwrap()
        .stiffness(&law)
        .unwrap();
    let rotated = Tet4Element::from_coordinates(moved)
        .unwrap()
        .stiffness(&law)
        .unwrap();
    let transform = ElementStiffness::from_fn(|i, j| {
        if i / 3 == j / 3 {
            q[(i % 3, j % 3)]
        } else {
            0.0
        }
    });
    assert!(
        (rotated - transform * original * transform.transpose()).norm() / original.norm() < 1e-12
    );
}

#[test]
fn invalid_geometry_and_nonfinite_displacements_return_errors() {
    let mut coordinates = cases().remove(0).coordinates;
    coordinates.swap(1, 2);
    assert!(matches!(
        Tet4Element::from_coordinates(coordinates),
        Err(FemError::Mesh(_))
    ));
    coordinates[3] = coordinates[0];
    assert!(Tet4Element::from_coordinates(coordinates).is_err());
    let element = Tet4Element::from_coordinates(cases().remove(0).coordinates).unwrap();
    assert!(
        element
            .strain(&ElementDisplacements::repeat(f64::NAN))
            .is_err()
    );
    assert!(
        element
            .strain(&ElementDisplacements::repeat(f64::INFINITY))
            .is_err()
    );
}

#[test]
fn unrepresentable_stiffness_and_energy_are_explicit_errors() {
    let coordinates = cases().remove(0).coordinates;
    let small = Tet4Element::from_coordinates(coordinates.map(|p| p.map(|v| v * 1e-100))).unwrap();
    let weak = IsotropicElastic::new(1e-300, 0.3).unwrap();
    assert!(matches!(
        small.stiffness(&weak),
        Err(FemError::NumericRange(_))
    ));
    let large = Tet4Element::from_coordinates(coordinates.map(|p| p.map(|v| v * 1e100))).unwrap();
    let strong = IsotropicElastic::new(1e300, 0.3).unwrap();
    assert!(matches!(
        large.stiffness(&strong),
        Err(FemError::NumericRange(_))
    ));
    let element = Tet4Element::from_coordinates(coordinates).unwrap();
    assert_eq!(
        element
            .response(&strong, &ElementDisplacements::zeros())
            .unwrap()
            .energy,
        0.0
    );
    let law = IsotropicElastic::new(1e-100, 0.3).unwrap();
    let u = affine(
        &coordinates,
        &(Matrix3::identity() * 1e-150),
        Vector3::zeros(),
    );
    assert!(matches!(
        element.response(&law, &u),
        Err(FemError::NumericRange(_))
    ));
}
