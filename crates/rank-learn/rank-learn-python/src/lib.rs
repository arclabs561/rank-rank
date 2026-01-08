//! Python bindings for rank-learn.
//!
//! Exposes LambdaRank, NDCG, and Neural LTR functionality to Python.

use ::rank_learn::lambdarank::{ndcg_at_k, LambdaRankParams, LambdaRankTrainer};
use pyo3::prelude::*;

/// LambdaRank parameters for Python.
#[pyclass]
#[derive(Clone, Copy)]
pub struct LambdaRankParamsPy {
    #[pyo3(get, set)]
    /// Sigmoid parameter (σ) for pairwise loss.
    /// Controls the sharpness of the sigmoid.
    /// Default: 1.0
    pub sigma: f32,
}

#[pymethods]
impl LambdaRankParamsPy {
    #[new]
    #[pyo3(signature = (sigma = 1.0))]
    fn new(sigma: f32) -> Self {
        Self { sigma }
    }

    fn __repr__(&self) -> String {
        format!("LambdaRankParams(sigma={})", self.sigma)
    }
}

impl From<LambdaRankParamsPy> for LambdaRankParams {
    fn from(params: LambdaRankParamsPy) -> Self {
        let mut result = LambdaRankParams::default();
        result.sigma = params.sigma;
        result
    }
}

/// LambdaRank trainer for Python.
#[pyclass]
pub struct LambdaRankTrainerPy {
    inner: LambdaRankTrainer,
}

#[pymethods]
impl LambdaRankTrainerPy {
    #[new]
    #[pyo3(signature = (params = None))]
    fn new(params: Option<LambdaRankParamsPy>) -> Self {
        let rust_params = params
            .map(|p| p.into())
            .unwrap_or_else(LambdaRankParams::default);
        Self {
            inner: LambdaRankTrainer::new(rust_params),
        }
    }

    /// Compute LambdaRank gradients for a query-document list.
    ///
    /// # Arguments
    ///
    /// * `scores` - Model scores for documents (list of floats)
    /// * `relevance` - Ground truth relevance scores (list of floats)
    /// * `k` - Optional NDCG@k to optimize (None = all positions)
    ///
    /// # Returns
    ///
    /// List of lambda values (gradients) for each document
    ///
    /// # Raises
    ///
    /// ValueError if scores or relevance is empty, or if lengths don't match.
    fn compute_gradients(
        &self,
        scores: Vec<f32>,
        relevance: Vec<f32>,
        k: Option<usize>,
    ) -> PyResult<Vec<f32>> {
        self.inner
            .compute_gradients(&scores, &relevance, k)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("LambdaRankTrainer()")
    }
}

/// Compute NDCG at a given position.
///
/// # Arguments
///
/// * `relevance` - Relevance scores for documents (in ranked order)
/// * `k` - Optional position to compute NDCG@k (None = all positions)
///
/// # Returns
///
/// NDCG value (float in [0, 1])
///
/// # Raises
///
/// ValueError if relevance is empty or k > relevance length.
#[pyfunction]
#[pyo3(signature = (relevance, k = None, exponential_gain = true))]
fn ndcg_at_k_py(relevance: Vec<f32>, k: Option<usize>, exponential_gain: bool) -> PyResult<f32> {
    ndcg_at_k(&relevance, k, exponential_gain)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

/// Python module for rank-learn.
#[pymodule]
fn rank_learn(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LambdaRankParamsPy>()?;
    m.add_class::<LambdaRankTrainerPy>()?;
    m.add_function(wrap_pyfunction!(ndcg_at_k_py, m)?)?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
