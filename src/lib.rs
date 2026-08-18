use pyo3::prelude::*;

#[pyfunction]
fn runtime(config:&str) ->anyhow::Result<()> {
    Ok(())
}

#[pymodule]
fn _taurino(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(runtime, m)?)?;
    Ok(())
}