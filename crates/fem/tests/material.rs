//! Analytic constitutive checks, including engineering-shear conventions.

use ferrasim_fem::{FemError, IsotropicElastic, MaterialWarning, VoigtVector};

#[test]
fn shear_and_uniaxial_response_match_analytic_solutions() {
    for poisson in [-0.8, -0.2, 0.0, 0.3, 0.48] {
        let young = 210e9;
        let law = IsotropicElastic::new(young, poisson).unwrap();
        let axial = VoigtVector::from_row_slice(&[
            0.001,
            -poisson * 0.001,
            -poisson * 0.001,
            0.0,
            0.0,
            0.0,
        ]);
        let actual = law.stress(&axial).unwrap();
        let expected = VoigtVector::from_row_slice(&[young * 0.001, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert!((actual - expected).norm() / expected.norm() < 1e-12);
        for axis in 3..6 {
            let mut strain = VoigtVector::zeros();
            strain[axis] = 0.02;
            let mut expected = VoigtVector::zeros();
            expected[axis] = young / (2.0 * (1.0 + poisson)) * 0.02;
            assert!((law.stress(&strain).unwrap() - expected).norm() / expected.norm() < 1e-12);
        }
    }
}

#[test]
fn volumetric_response_matches_bulk_modulus() {
    for poisson in [-0.6, 0.0, 0.3, 0.49] {
        let law = IsotropicElastic::new(1000.0, poisson).unwrap();
        let strain = VoigtVector::from_row_slice(&[0.001, 0.001, 0.001, 0.0, 0.0, 0.0]);
        let pressure = 1000.0 / (1.0 - 2.0 * poisson) * 0.001;
        let expected = VoigtVector::from_row_slice(&[pressure, pressure, pressure, 0.0, 0.0, 0.0]);
        assert!((law.stress(&strain).unwrap() - expected).norm() / expected.norm() < 1e-12);
    }
}

#[test]
fn invalid_constants_and_unrepresentable_tensors_are_errors() {
    for (young, poisson) in [
        (0.0, 0.3),
        (-1.0, 0.3),
        (f64::NAN, 0.3),
        (f64::INFINITY, 0.3),
        (1.0, -1.0),
        (1.0, 0.5),
        (1.0, f64::NAN),
    ] {
        assert!(matches!(
            IsotropicElastic::new(young, poisson),
            Err(FemError::InvalidMaterial(_))
        ));
    }
    assert!(matches!(
        IsotropicElastic::new(f64::MAX, 0.49),
        Err(FemError::NumericRange(_))
    ));
    let law = IsotropicElastic::new(100.0, 0.3).unwrap();
    assert!(law.stress(&VoigtVector::repeat(f64::INFINITY)).is_err());
    assert!(law.stress(&VoigtVector::repeat(f64::MAX)).is_err());
}

#[test]
fn near_incompressibility_is_advisory_and_never_clamps_parameters() {
    assert!(
        IsotropicElastic::new(1.0, 0.489)
            .unwrap()
            .warnings()
            .is_empty()
    );
    let law = IsotropicElastic::new(1.0, 0.49).unwrap();
    assert_eq!(law.poisson(), 0.49);
    assert_eq!(
        law.warnings(),
        vec![MaterialWarning::NearIncompressible { poisson: 0.49 }]
    );
    let auxetic = IsotropicElastic::new(1.0, -0.2).unwrap();
    assert!(auxetic.lambda() < 0.0);
    assert!(auxetic.matrix().cholesky().is_some());
}

#[test]
fn finite_tiny_units_do_not_overflow_the_normalization_reciprocal() {
    let law = IsotropicElastic::new(1e-310, 0.3).unwrap();
    assert!(law.matrix().iter().all(|value| value.is_finite()));
    assert!(law.stress(&VoigtVector::repeat(1e-310)).is_err());
    let zero = law.stress(&VoigtVector::zeros()).unwrap();
    assert_eq!(zero, VoigtVector::zeros());
}
