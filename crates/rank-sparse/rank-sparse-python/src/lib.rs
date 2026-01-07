use pyo3::prelude::*;
use rank_sparse::{SparseVector as RsSparseVector, dot_product as rs_dot_product};

#[pyclass]
#[derive(Clone)]
struct SparseVector {
    inner: RsSparseVector,
}

#[pymethods]
impl SparseVector {
    #[new]
    fn new(indices: Vec<u32>, values: Vec<f32>) -> PyResult<Self> {
        match RsSparseVector::new(indices, values) {
            Some(v) => Ok(SparseVector { inner: v }),
            None => Err(pyo3::exceptions::PyValueError::new_err(
                "Indices must be same length as values and sorted strictly increasing.",
            )),
        }
    }

    fn prune(&self, threshold: f32) -> Self {
        SparseVector {
            inner: self.inner.prune(threshold),
        }
    }
    
    fn __repr__(&self) -> String {
        format!("SparseVector(indices={:?}, values={:?})", self.inner.indices, self.inner.values)
    }
}

#[pyfunction]
fn dot_product(a: &SparseVector, b: &SparseVector) -> f32 {
    rs_dot_product(&a.inner, &b.inner)
}

#[pymodule]
fn rank_sparse(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SparseVector>()?;
    m.add_function(wrap_pyfunction!(dot_product, m)?)?;
    Ok(())
}
