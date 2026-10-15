use ferrasim_model::Material;

use crate::{ElasticityMatrix, FemError, VoigtVector};

/// Initial advisory threshold for TET4 volumetric locking; not a validity bound.
pub const NEAR_INCOMPRESSIBLE_THRESHOLD: f64 = 0.49;

/// Material limitations requiring explicit user-facing diagnostics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialWarning {
    /// Displacement-only TET4 is not qualified for near-incompressible response.
    NearIncompressible {
        /// Material Poisson's ratio that triggered the advisory.
        poisson: f64,
    },
}

/// Immutable, finite isotropic elastic operator for infinitesimal strains.
#[derive(Debug, Clone)]
pub struct IsotropicElastic {
    young: f64,
    poisson: f64,
    lambda: f64,
    shear: f64,
    bulk: f64,
    elasticity: ElasticityMatrix,
}

impl IsotropicElastic {
    /// Construct and check physical ranges and representability of the tensor.
    pub fn new(young: f64, poisson: f64) -> Result<Self, FemError> {
        if !young.is_finite() || young <= 0.0 {
            return Err(FemError::InvalidMaterial(
                "Young's modulus must be finite and positive",
            ));
        }
        if !(poisson.is_finite() && -1.0 < poisson && poisson < 0.5) {
            return Err(FemError::InvalidMaterial(
                "Poisson's ratio must be finite and satisfy -1 < nu < 0.5",
            ));
        }
        let shear = young / (2.0 * (1.0 + poisson));
        let bulk = young / (3.0 * (1.0 - 2.0 * poisson));
        let lambda = young * (poisson / (1.0 + poisson) / (1.0 - 2.0 * poisson));
        let normal = young * ((1.0 - poisson) / (1.0 + poisson) / (1.0 - 2.0 * poisson));
        let mut elasticity = ElasticityMatrix::zeros();
        for i in 0..3 {
            for j in 0..3 {
                elasticity[(i, j)] = if i == j { normal } else { lambda };
            }
            elasticity[(i + 3, i + 3)] = shear;
        }
        if ![shear, bulk, lambda].iter().all(|v| v.is_finite())
            || shear <= 0.0
            || bulk <= 0.0
            || elasticity.iter().any(|v| !v.is_finite())
        {
            return Err(FemError::NumericRange(
                "elastic coefficients are not representable",
            ));
        }
        // Scale before factorization so subnormal/large physical units do not
        // alone determine the numerical positive-definiteness check.
        let scale = elasticity.amax();
        let scaled = elasticity.map(|value| value / scale);
        if scaled.cholesky().is_none() {
            return Err(FemError::NumericRange(
                "elastic tensor is not positive definite at working precision",
            ));
        }
        Ok(Self {
            young,
            poisson,
            lambda,
            shear,
            bulk,
            elasticity,
        })
    }

    /// Young's modulus in the declared coherent units.
    pub fn young(&self) -> f64 {
        self.young
    }
    /// Poisson's ratio.
    pub fn poisson(&self) -> f64 {
        self.poisson
    }
    /// First Lamé coefficient, which may be negative for auxetic materials.
    pub fn lambda(&self) -> f64 {
        self.lambda
    }
    /// Positive shear modulus.
    pub fn shear_modulus(&self) -> f64 {
        self.shear
    }
    /// Positive bulk modulus.
    pub fn bulk_modulus(&self) -> f64 {
        self.bulk
    }
    /// Immutable constitutive matrix with engineering-shear convention.
    pub fn matrix(&self) -> &ElasticityMatrix {
        &self.elasticity
    }

    /// Advisory warnings; no constitutive parameter is modified automatically.
    pub fn warnings(&self) -> Vec<MaterialWarning> {
        if self.poisson >= NEAR_INCOMPRESSIBLE_THRESHOLD {
            vec![MaterialWarning::NearIncompressible {
                poisson: self.poisson,
            }]
        } else {
            Vec::new()
        }
    }

    /// Evaluate sigma = C epsilon for a finite engineering-strain vector.
    pub fn stress(&self, strain: &VoigtVector) -> Result<VoigtVector, FemError> {
        if strain.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange("strain must be finite"));
        }
        let stress = self.elasticity * strain;
        if stress.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange("stress overflow"));
        }
        if strain.amax() > 0.0 && stress.amax() == 0.0 {
            return Err(FemError::NumericRange("stress underflow"));
        }
        Ok(stress)
    }
}

impl TryFrom<&Material> for IsotropicElastic {
    type Error = FemError;
    fn try_from(material: &Material) -> Result<Self, Self::Error> {
        match material {
            Material::IsotropicLinearElastic { young, poisson, .. } => Self::new(*young, *poisson),
        }
    }
}
