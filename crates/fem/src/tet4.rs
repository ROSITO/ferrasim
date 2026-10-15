use ferrasim_mesh::{ElementId, Mesh, Tet4, ValidatedMesh};
use ferrasim_model::NodeId;
use nalgebra::{Matrix3, SMatrix, Vector3};

use crate::{
    ElementDisplacements, ElementStiffness, FemError, IsotropicElastic, StrainDisplacementMatrix,
    VoigtVector,
};

/// Element-local constant strain, stress and integrated elastic energy.
#[derive(Debug, Clone)]
pub struct ElementResponse {
    /// Engineering strain in order xx, yy, zz, xy, yz, xz.
    pub strain: VoigtVector,
    /// Cauchy stress in order xx, yy, zz, xy, yz, xz.
    pub stress: VoigtVector,
    /// Integrated strain energy in force times length units.
    pub energy: f64,
}

/// Immutable affine TET4 geometry and constant kinematic operators.
#[derive(Debug, Clone)]
pub struct Tet4Element {
    coordinates: [[f64; 3]; 4],
    jacobian: Matrix3<f64>,
    gradients: SMatrix<f64, 3, 4>,
    b: StrainDisplacementMatrix,
    b_normalized: StrainDisplacementMatrix,
    length: f64,
    determinant_normalized: f64,
    volume: f64,
}

fn b_matrix(gradients: &SMatrix<f64, 3, 4>) -> StrainDisplacementMatrix {
    let mut b = StrainDisplacementMatrix::zeros();
    for node in 0..4 {
        let x = gradients[(0, node)];
        let y = gradients[(1, node)];
        let z = gradients[(2, node)];
        let col = 3 * node;
        b[(0, col)] = x;
        b[(1, col + 1)] = y;
        b[(2, col + 2)] = z;
        b[(3, col)] = y;
        b[(3, col + 1)] = x;
        b[(4, col + 1)] = z;
        b[(4, col + 2)] = y;
        b[(5, col)] = z;
        b[(5, col + 2)] = x;
    }
    b
}

impl Tet4Element {
    /// Construct from coordinates, applying the same geometric policy as S02.
    pub fn from_coordinates(coordinates: [[f64; 3]; 4]) -> Result<Self, FemError> {
        let mut mesh = Mesh::default();
        for (i, point) in coordinates.into_iter().enumerate() {
            mesh.nodes.insert(NodeId(i as u64 + 1), point);
        }
        let id = ElementId(1);
        mesh.tetrahedra.insert(
            id,
            Tet4 {
                id,
                nodes: [NodeId(1), NodeId(2), NodeId(3), NodeId(4)],
                physical_tag: None,
                entity_tag: None,
            },
        );
        Self::from_mesh(&mesh.into_validated()?, id)
    }

    /// Construct from one existing tetrahedron, preserving external node order.
    pub fn from_mesh(mesh: &ValidatedMesh, id: ElementId) -> Result<Self, FemError> {
        let tet = mesh
            .mesh()
            .tetrahedra
            .get(&id)
            .ok_or(FemError::MissingElement(id))?;
        let coordinates = tet.nodes.each_ref().map(|node| mesh.mesh().nodes[node]);
        let origin = Vector3::from(coordinates[0]);
        let columns: [Vector3<f64>; 3] =
            std::array::from_fn(|i| Vector3::from(coordinates[i + 1]) - origin);
        let jacobian = Matrix3::from_columns(&columns);
        let mut length: f64 = 0.0;
        for i in 0..4 {
            for j in i + 1..4 {
                let d = std::array::from_fn::<_, 3, _>(|k| coordinates[i][k] - coordinates[j][k]);
                length = length.max(d[0].hypot(d[1]).hypot(d[2]));
            }
        }
        let inverse_transpose = (jacobian / length)
            .try_inverse()
            .ok_or(FemError::NumericRange(
                "normalized Jacobian is not invertible",
            ))?
            .transpose();
        let reference = SMatrix::<f64, 3, 4>::from_row_slice(&[
            -1.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0,
        ]);
        let gradients_normalized = inverse_transpose * reference;
        let gradients = gradients_normalized / length;
        let b = b_matrix(&gradients);
        if b.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange(
                "shape gradients are not representable",
            ));
        }
        let geometry = &mesh.geometry()[&id];
        Ok(Self {
            coordinates,
            jacobian,
            gradients,
            b,
            b_normalized: b_matrix(&gradients_normalized),
            length,
            determinant_normalized: geometry.normalized_determinant,
            volume: geometry.volume,
        })
    }

    /// Original vertex coordinates in local connectivity order.
    pub fn coordinates(&self) -> &[[f64; 3]; 4] {
        &self.coordinates
    }
    /// Constant physical Jacobian, with edge vectors in columns.
    pub fn jacobian(&self) -> &Matrix3<f64> {
        &self.jacobian
    }
    /// Constant global gradients, one column per shape function.
    pub fn gradients(&self) -> &SMatrix<f64, 3, 4> {
        &self.gradients
    }
    /// Constant strain-displacement matrix, using engineering shear strains.
    pub fn b_matrix(&self) -> &StrainDisplacementMatrix {
        &self.b
    }
    /// Positive geometric volume.
    pub fn volume(&self) -> f64 {
        self.volume
    }

    /// Affine shape functions at finite reference coordinates (extrapolation allowed).
    pub fn shape_functions(reference: [f64; 3]) -> Result<[f64; 4], FemError> {
        let [r, s, t] = reference;
        let values = [1.0 - r - s - t, r, s, t];
        if values.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange(
                "reference shape values must be finite",
            ));
        }
        Ok(values)
    }

    /// Compute the full raw 12x12 stiffness, without artificial symmetrization.
    ///
    /// Normalized operators avoid forming dimensional B² before multiplying by
    /// volume. Nonfinite or wholly underflowed results are reported as errors.
    pub fn stiffness(&self, material: &IsotropicElastic) -> Result<ElementStiffness, FemError> {
        let scale = material.matrix().amax();
        let normalized = self.b_normalized.transpose()
            * material.matrix().map(|value| value / scale)
            * self.b_normalized;
        let stiffness = normalized
            .map(|entry| ((entry * (self.determinant_normalized / 6.0)) * self.length) * scale);
        if stiffness.iter().any(|v| !v.is_finite()) || stiffness.amax() == 0.0 {
            return Err(FemError::NumericRange(
                "element stiffness is not representable",
            ));
        }
        Ok(stiffness)
    }

    /// Evaluate constant engineering strain from finite local displacements.
    pub fn strain(&self, displacements: &ElementDisplacements) -> Result<VoigtVector, FemError> {
        if displacements.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange("displacements must be finite"));
        }
        let strain = self.b * displacements;
        if strain.iter().any(|v| !v.is_finite()) {
            return Err(FemError::NumericRange("strain overflow"));
        }
        Ok(strain)
    }

    /// Evaluate local strain, stress and elastic energy; this does not solve Ku=F.
    pub fn response(
        &self,
        material: &IsotropicElastic,
        displacements: &ElementDisplacements,
    ) -> Result<ElementResponse, FemError> {
        let strain = self.strain(displacements)?;
        let stress = material.stress(&strain)?;
        let energy = (strain.dot(&stress) / 2.0) * self.volume;
        if !energy.is_finite() || energy < 0.0 || (strain.amax() > 0.0 && energy == 0.0) {
            return Err(FemError::NumericRange(
                "elastic energy is not representable as a nonnegative finite value",
            ));
        }
        Ok(ElementResponse {
            strain,
            stress,
            energy,
        })
    }
}
