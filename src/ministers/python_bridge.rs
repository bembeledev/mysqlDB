/*use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::ffi::CString;

pub fn init_python_engine() -> Result<(), String> {
    // pyo3 auto-initializes with the 'auto-initialize' feature enabled
    println!("Python engine initialized.");
    Ok(())
}

pub fn eval_py_block(content: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    // 1. Mudança de with_gil para attach
    Python::attach(|py| {
        // 2. Mudança de new_bound para apenas new
        let locals = PyDict::new(py);

        // Inserir variáveis no escopo Python
        for (k, v) in vars {
            if let Ok(n) = v.parse::<i64>() {
                let _ = locals.set_item(k, n);
            } else if let Ok(f) = v.parse::<f64>() {
                let _ = locals.set_item(k, f);
            } else if v == "true" {
                let _ = locals.set_item(k, true);
            } else if v == "false" {
                let _ = locals.set_item(k, false);
            } else {
                let _ = locals.set_item(k, v);
            }
        }

        let wrapped_code = format!(
            "def __polyglot_exec():\n{}\n\n__polyglot_result = __polyglot_exec()",
            content.lines().map(|l| format!("    {}", l)).collect::<Vec<String>>().join("\n")
        );

        // 3. Converter para CString (exigido pelo novo método run)
        let code_cstr = CString::new(wrapped_code).map_err(|e| e.to_string())?;

        // 4. Mudança de run_bound para apenas run
        match py.run(&code_cstr, None, Some(&locals)) {
            Ok(_) => {
                if let Ok(Some(res)) = locals.get_item("__polyglot_result") {
                    if res.is_none() {
                        return Ok("void".to_string());
                    }
                    if let Ok(n) = res.extract::<f64>() {
                        return Ok(n.to_string());
                    }
                    if let Ok(b) = res.extract::<bool>() {
                        return Ok(b.to_string());
                    }
                    if let Ok(s) = res.extract::<String>() {
                        return Ok(s);
                    }
                    Ok("object".to_string())
                } else {
                    Ok("void".to_string())
                }
            }
            Err(e) => Err(format!("Python FFI Error: {:?}", e)),
        }
    })
}*/