use crate::framing;
use pyo3::prelude::*;

mod gaf;
mod gam;
mod gamp;
mod dict;

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

#[pymodule]
fn gax(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_submodule(gaf::submodule(py)?)?;
    m.add_submodule(gam::submodule(py)?)?;
    m.add_submodule(gamp::submodule(py)?)?;
    Ok(())
}
