use axum::http::{HeaderMap, HeaderValue};
use pyo3::types::IntoPyDict;
use pyo3::{prelude::*, types::PyList, types::PyModule, types::PyTuple};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn get_headers_map(headers: &HeaderMap<HeaderValue>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_owned();
        let value_str = String::from_utf8_lossy(value.as_bytes()).into_owned();
        map.insert(key_str, value_str);
    }
    map
}

pub fn call_python(
    invokation_path: &str,
    query_params: HashMap<String, String>,
    path_params: Vec<i32>,
    headers: HeaderMap,
    body: String,
) -> Result<Py<PyAny>, PyErr> {
    // TODO: handle args
    let mut l = invokation_path.split(":");
    println!("invoking python...");
    let path = Path::new(l.next().unwrap());
    let py_app = fs::read_to_string(path.join(l.next().unwrap()))?;
    Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        let syspath = py
            .import_bound("sys")?
            .getattr("path")?
            .downcast_into::<PyList>()?;
        syspath.insert(0, &path)?;
        let view: Py<PyAny> = PyModule::from_code_bound(py, &py_app, "", "")?
            .getattr(l.next().unwrap())?
            .into();
        /*let kwargs = HashMap::new();
        kwargs.insert("query_params", query_params);
        kwargs.insert("headers", get_headers_map(&headers));*/
        //kwargs.insert("body", body);
        // TODO: handle multi-level json's
        let payload = match body.is_empty() {
            true => HashMap::new(),
            false => serde_json::from_str::<HashMap<String, String>>(&body).unwrap(),
        };

        println!("payload: {:?}", payload);

        let mut kwargs = HashMap::new();
        kwargs.insert("payload", payload);
        kwargs.insert("query_params", query_params);
        kwargs.insert("headers", get_headers_map(&headers));
        view.call_bound(
            py,
            PyTuple::new_bound(py, path_params),
            Some(&kwargs.into_py_dict_bound(py)),
        )
        /*view.call1(
            py,
            (query_params, path_params, get_headers_map(&headers), body),
        )*/
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_python() {
        //let r = call_python("examples/music:songs.py:list");
        //println!("r: {:?}", r.unwrap().to_string());
    }
}
