use crate::{framing, gaf::GafRecord, vg};
use graph::GFAWrapper;
use pyo3::prelude::*;

mod dict;
mod gaf;
mod gam;
mod gamp;
mod graph;

impl From<framing::Error> for PyErr {
    fn from(e: framing::Error) -> Self {
        match e {
            framing::Error::Io(e) => e.into(),
            framing::Error::Utf8(e) => e.into(),
            framing::Error::ProstDecode(e) => {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Error: {}", e))
            }
            framing::Error::ProstEncode(e) => {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Error: {}", e))
            }
            framing::Error::InvalidTypeTag(_, _) => {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Error: {}", e))
            }
        }
    }
}

#[pyfunction]
pub fn convert_gam_to_gaf(value: Vec<PyObject>, graph: &GFAWrapper) -> PyResult<Vec<PyObject>> {
    Python::with_gil(|py| -> PyResult<_> {
        let gam = value
            .iter()
            .map(|o| -> PyResult<_> { o.extract::<vg::Alignment>(py) })
            .collect::<PyResult<Vec<_>>>()?;
        let gaf = crate::convert_gam_to_gaf(&gam, &graph.graph);
        let py_gaf = gaf.iter().map(|o| o.clone().into_py(py)).collect();
        Ok(py_gaf)
    })
}

#[pyfunction]
pub fn convert_gaf_to_gam(value: Vec<PyObject>, graph: &GFAWrapper) -> PyResult<Vec<PyObject>> {
    Python::with_gil(|py| -> PyResult<_> {
        let gaf = value
            .iter()
            .map(|o| -> PyResult<_> { o.extract::<GafRecord>(py) })
            .collect::<PyResult<Vec<_>>>()?;
        let gam = crate::convert_gaf_to_gam(&gaf, &graph.graph);
        let py_gam = gam.iter().map(|o| o.clone().into_py(py)).collect();
        Ok(py_gam)
    })
}

#[pymodule]
fn gax(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_submodule(gaf::submodule(py)?)?;
    m.add_submodule(gam::submodule(py)?)?;
    m.add_submodule(gamp::submodule(py)?)?;
    m.add_function(wrap_pyfunction!(graph::load_graph, m)?)?;
    m.add_function(wrap_pyfunction!(convert_gam_to_gaf, m)?)?;
    m.add_function(wrap_pyfunction!(convert_gaf_to_gam, m)?)?;
    Ok(())
}
