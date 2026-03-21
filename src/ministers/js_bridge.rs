use boa_engine::{Context, JsValue};
use std::collections::HashMap;

pub fn init_js_engine() -> Result<(), String> {
    println!("JS engine initialized.");
    Ok(())
}

pub fn eval_js_block(content: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut context = Context::default();

    for (k, v) in vars {
        let js_val = if let Ok(n) = v.parse::<f64>() {
            JsValue::from(n)
        } else if v == "true" {
            JsValue::from(true)
        } else if v == "false" {
            JsValue::from(false)
        } else {
            JsValue::from(boa_engine::JsString::from(v.as_str()))
        };
        let _ = context.register_global_property(boa_engine::JsString::from(k.as_str()), js_val, boa_engine::property::Attribute::all());
    }

    let wrapped_code = format!("(() => {{\n{}\n}})()", content);

    match context.eval(boa_engine::Source::from_bytes(wrapped_code.as_bytes())) {
        Ok(res) => {
            if res.is_undefined() || res.is_null() {
                Ok("void".to_string())
            } else if let Some(n) = res.as_number() {
                Ok(n.to_string())
            } else if let Some(b) = res.as_boolean() {
                Ok(b.to_string())
            } else if let Some(s) = res.as_string() {
                Ok(s.to_std_string_escaped())
            } else {
                Ok("object".to_string())
            }
        }
        Err(e) => Err(format!("JS FFI Error: {:?}", e)),
    }
}
