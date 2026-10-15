//! Checked CSR operators, CG/Jacobi and explicit convergence/failure reports.

mod cg;
pub use cg::{
    CG_SYMMETRY_TOLERANCE, SolveError, SolveFailureReason, SolveReport, SolveStatus,
    solve_cg_jacobi,
};

use nalgebra_sparse::CsrMatrix;
use std::collections::BTreeMap;
use thiserror::Error;

/// Compensate both product rounding (FMA) and cancellation in a dot product.
#[derive(Default)]
struct AccurateDot {
    sum: f64,
    correction: f64,
}
impl AccurateDot {
    fn add(&mut self, a: f64, b: f64) -> Option<()> {
        let product = a * b;
        let next = self.sum + product;
        if !product.is_finite() || !next.is_finite() {
            return None;
        }
        let rounding = if self.sum.abs() >= product.abs() {
            (self.sum - next) + product
        } else {
            (product - next) + self.sum
        };
        self.correction += rounding + a.mul_add(b, -product);
        self.sum = next;
        self.correction.is_finite().then_some(())
    }
    fn finish(self) -> Option<f64> {
        let value = self.sum + self.correction;
        value.is_finite().then_some(value)
    }
}

/// Invalid dimensions, indices or floating-point arithmetic.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid sparse operation: {0}")]
pub struct SparseError(pub &'static str);

/// Immutable finite square CSR matrix, with sorted unique columns per row.
#[derive(Debug, Clone)]
pub struct SparseMatrix(CsrMatrix<f64>);

impl SparseMatrix {
    /// Sum duplicate triplets in input order; reject out-of-range or nonfinite data.
    /// Exact zeros are omitted. Storage and work never require a dense matrix.
    pub fn from_triplets(
        size: usize,
        entries: impl IntoIterator<Item = (usize, usize, f64)>,
    ) -> Result<Self, SparseError> {
        let mut sums = BTreeMap::new();
        for (row, col, value) in entries {
            if row >= size || col >= size || !value.is_finite() {
                return Err(SparseError("triplet index or value"));
            }
            let sum = sums.entry((row, col)).or_insert(0.0);
            *sum += value;
            if !sum.is_finite() {
                return Err(SparseError("duplicate accumulation overflow"));
            }
        }
        let mut offsets = vec![
            0;
            size.checked_add(1)
                .ok_or(SparseError("dimension overflow"))?
        ];
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for ((row, col), value) in sums {
            if value != 0.0 {
                offsets[row + 1] += 1;
                columns.push(col);
                values.push(value);
            }
        }
        for row in 0..size {
            offsets[row + 1] += offsets[row];
        }
        CsrMatrix::try_from_csr_data(size, size, offsets, columns, values)
            .map(Self)
            .map_err(|_| SparseError("invalid CSR structure"))
    }

    /// Number of rows and columns.
    pub fn size(&self) -> usize {
        self.0.nrows()
    }
    /// Read-only CSR representation for sparse solver integration.
    pub fn csr(&self) -> &CsrMatrix<f64> {
        &self.0
    }
    /// Stored entries in deterministic row-major order.
    pub fn entries(&self) -> impl Iterator<Item = (usize, usize, f64)> + '_ {
        (0..self.size()).flat_map(move |row| {
            (self.0.row_offsets()[row]..self.0.row_offsets()[row + 1])
                .map(move |index| (row, self.0.col_indices()[index], self.0.values()[index]))
        })
    }
    /// Sparse matrix-vector product with checked input and arithmetic.
    pub fn multiply(&self, vector: &[f64]) -> Result<Vec<f64>, SparseError> {
        if vector.len() != self.size() || vector.iter().any(|x| !x.is_finite()) {
            return Err(SparseError("vector dimension or value"));
        }
        let mut result = vec![0.0; self.size()];
        for (row, result) in result.iter_mut().enumerate() {
            let mut dot = AccurateDot::default();
            for index in self.0.row_offsets()[row]..self.0.row_offsets()[row + 1] {
                dot.add(self.0.values()[index], vector[self.0.col_indices()[index]])
                    .ok_or(SparseError("matrix-vector product overflow"))?;
            }
            *result = dot
                .finish()
                .ok_or(SparseError("matrix-vector product overflow"))?;
        }
        Ok(result)
    }
}
