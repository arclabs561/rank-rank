//! Python bindings for rank-retrieve.
//!
//! Provides a Python API that mirrors the Rust API, enabling seamless
//! integration with Python RAG/search stacks.
//!
//! # Usage
//!
//! ```python
//! import rank_retrieve
//!
//! # BM25 retrieval
//! index = rank_retrieve.InvertedIndex()
//! index.add_document(0, ["the", "quick", "brown"])
//! results = index.retrieve(["quick"], 10)
//!
//! # Dense retrieval
//! retriever = rank_retrieve.DenseRetriever()
//! retriever.add_document(0, [1.0, 0.0, 0.0])
//! results = retriever.retrieve([1.0, 0.0, 0.0], 10)
//!
//! # Sparse retrieval
//! retriever = rank_retrieve.SparseRetriever()
//! vector = rank_retrieve.SparseVector([0, 1, 2], [1.0, 0.5, 0.3])
//! retriever.add_document(0, vector)
//! results = retriever.retrieve(vector, 10)
//! ```

// Note: allow(deprecated) needed for pyo3 0.24 compatibility
// TODO: Remove when upgrading to pyo3 0.25+ which uses IntoPyObject
// Impact: This suppresses deprecation warnings for pyo3's IntoPy trait methods.
// Action: Check pyo3 changelog when upgrading to 0.25+ to see if IntoPyObject migration is needed.
#![allow(deprecated)]

use ::rank_retrieve::{
    bm25::{Bm25Params, InvertedIndex},
    dense::DenseRetriever,
    sparse::SparseRetriever,
    sparse::{SparseVector as RustSparseVector, dot_product},
};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::PyRef;

/// Python module for rank-retrieve.
#[pymodule]
#[pyo3(name = "rank_retrieve")]
fn rank_retrieve_module(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // BM25 classes
    m.add_class::<InvertedIndexPy>()?;
    m.add_class::<Bm25ParamsPy>()?;

    // Dense retrieval classes
    m.add_class::<DenseRetrieverPy>()?;

    // Sparse retrieval classes
    m.add_class::<SparseRetrieverPy>()?;
    m.add_class::<SparseVectorPy>()?;

    // Utility functions
    m.add_function(wrap_pyfunction!(sparse_dot_product_py, m)?)?;

    Ok(())
}

/// Python wrapper for InvertedIndex.
#[pyclass]
pub struct InvertedIndexPy {
    inner: InvertedIndex,
}

#[pymethods]
impl InvertedIndexPy {
    #[new]
    fn new() -> Self {
        Self {
            inner: InvertedIndex::new(),
        }
    }

    /// Add a document to the index.
    ///
    /// # Arguments
    /// * `doc_id` - Document identifier (u32)
    /// * `terms` - List of tokenized terms (strings)
    fn add_document(&mut self, doc_id: u32, terms: &Bound<'_, PyList>) -> PyResult<()> {
        let terms_vec: Vec<String> = terms.extract()?;
        self.inner.add_document(doc_id, &terms_vec);
        Ok(())
    }

    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    /// * `query_terms` - List of tokenized query terms (strings)
    /// * `k` - Number of documents to retrieve
    /// * `params` - Optional BM25 parameters (defaults used if None)
    ///
    /// # Returns
    /// List of (doc_id, score) tuples sorted by score descending
    #[pyo3(signature = (query_terms, k, params = None))]
    fn retrieve(
        &self,
        query_terms: &Bound<'_, PyList>,
        k: usize,
        params: Option<PyRef<'_, Bm25ParamsPy>>,
    ) -> PyResult<Vec<(u32, f32)>> {
        let query_vec: Vec<String> = query_terms.extract()?;
        let bm25_params = params
            .map(|p| p.inner)
            .unwrap_or_else(Bm25Params::default);

        self.inner
            .retrieve(&query_vec, k, bm25_params)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }

    /// Calculate IDF for a term.
    fn idf(&self, term: &str) -> f32 {
        self.inner.idf(term)
    }
}

/// Python wrapper for Bm25Params.
#[pyclass]
#[derive(Clone, Copy)]
pub struct Bm25ParamsPy {
    inner: Bm25Params,
}

#[pymethods]
impl Bm25ParamsPy {
    #[new]
    #[pyo3(signature = (k1 = 1.2, b = 0.75))]
    fn new(k1: f32, b: f32) -> Self {
        Self {
            inner: Bm25Params { k1, b },
        }
    }

    #[getter]
    fn k1(&self) -> f32 {
        self.inner.k1
    }

    #[getter]
    fn b(&self) -> f32 {
        self.inner.b
    }

    fn with_k1(&self, k1: f32) -> Self {
        Self {
            inner: Bm25Params {
                k1,
                b: self.inner.b,
            },
        }
    }

    fn with_b(&self, b: f32) -> Self {
        Self {
            inner: Bm25Params {
                k1: self.inner.k1,
                b,
            },
        }
    }
}

/// Python wrapper for DenseRetriever.
#[pyclass]
pub struct DenseRetrieverPy {
    inner: DenseRetriever,
}

#[pymethods]
impl DenseRetrieverPy {
    #[new]
    fn new() -> Self {
        Self {
            inner: DenseRetriever::new(),
        }
    }

    /// Add a document with its dense embedding.
    ///
    /// # Arguments
    /// * `doc_id` - Document identifier (u32)
    /// * `embedding` - Dense embedding vector (list of f32)
    fn add_document(&mut self, doc_id: u32, embedding: &Bound<'_, PyList>) -> PyResult<()> {
        let embedding_vec: Vec<f32> = embedding.extract()?;
        self.inner.add_document(doc_id, embedding_vec);
        Ok(())
    }

    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    /// * `query_embedding` - Query embedding vector (list of f32)
    /// * `k` - Number of documents to retrieve
    ///
    /// # Returns
    /// List of (doc_id, score) tuples sorted by score descending
    fn retrieve(
        &self,
        query_embedding: &Bound<'_, PyList>,
        k: usize,
    ) -> PyResult<Vec<(u32, f32)>> {
        let query_vec: Vec<f32> = query_embedding.extract()?;
        self.inner
            .retrieve(&query_vec, k)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }

    /// Score a document against a query.
    ///
    /// # Arguments
    /// * `doc_id` - Document identifier
    /// * `query_embedding` - Query embedding vector
    ///
    /// # Returns
    /// Cosine similarity score, or None if document not found
    fn score(
        &self,
        doc_id: u32,
        query_embedding: &Bound<'_, PyList>,
    ) -> PyResult<Option<f32>> {
        let query_vec: Vec<f32> = query_embedding.extract()?;
        Ok(self.inner.score(doc_id, &query_vec))
    }
}

/// Python wrapper for SparseVector.
#[pyclass]
#[derive(Clone)]
pub struct SparseVectorPy {
    inner: RustSparseVector,
}

#[pymethods]
impl SparseVectorPy {
    #[new]
    #[pyo3(signature = (indices, values, *, validate = true))]
    fn new(
        indices: Vec<u32>,
        values: Vec<f32>,
        validate: bool,
    ) -> PyResult<Self> {
        let vector = if validate {
            RustSparseVector::new(indices, values)
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Invalid sparse vector: indices and values must have same length, and indices must be sorted and unique",
                    )
                })?
        } else {
            RustSparseVector::new_unchecked(indices, values)
        };
        Ok(Self { inner: vector })
    }

    #[getter]
    fn indices(&self) -> Vec<u32> {
        self.inner.indices.clone()
    }

    #[getter]
    fn values(&self) -> Vec<f32> {
        self.inner.values.clone()
    }

    /// Prune the vector by removing values below threshold.
    fn prune(&self, threshold: f32) -> Self {
        Self {
            inner: self.inner.prune(threshold),
        }
    }
}

/// Python wrapper for SparseRetriever.
#[pyclass]
pub struct SparseRetrieverPy {
    inner: SparseRetriever,
}

#[pymethods]
impl SparseRetrieverPy {
    #[new]
    fn new() -> Self {
        Self {
            inner: SparseRetriever::new(),
        }
    }

    /// Add a document with its sparse vector representation.
    ///
    /// # Arguments
    /// * `doc_id` - Document identifier (u32)
    /// * `vector` - SparseVector instance
    fn add_document(&mut self, doc_id: u32, vector: PyRef<'_, SparseVectorPy>) -> PyResult<()> {
        self.inner.add_document(doc_id, vector.inner.clone());
        Ok(())
    }

    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    /// * `query_vector` - SparseVector instance
    /// * `k` - Number of documents to retrieve
    ///
    /// # Returns
    /// List of (doc_id, score) tuples sorted by score descending
    fn retrieve(
        &self,
        query_vector: PyRef<'_, SparseVectorPy>,
        k: usize,
    ) -> PyResult<Vec<(u32, f32)>> {
        self.inner
            .retrieve(&query_vector.inner, k)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }

    /// Score a document against a query.
    ///
    /// # Arguments
    /// * `doc_id` - Document identifier
    /// * `query_vector` - SparseVector instance
    ///
    /// # Returns
    /// Dot product score, or None if document not found
    fn score(
        &self,
        doc_id: u32,
        query_vector: PyRef<'_, SparseVectorPy>,
    ) -> PyResult<Option<f32>> {
        Ok(self.inner.score(doc_id, &query_vector.inner))
    }
}

/// Compute dot product between two sparse vectors.
#[pyfunction]
fn sparse_dot_product_py(
    a: PyRef<'_, SparseVectorPy>,
    b: PyRef<'_, SparseVectorPy>,
) -> f32 {
    dot_product(&a.inner, &b.inner)
}
