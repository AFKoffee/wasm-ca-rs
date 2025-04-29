use std::fmt::Display;

use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug)]
pub enum Error {
    JsError(String),
    InvalidHandle(String),
}

impl From<&JsValue> for Error {
    fn from(value: &JsValue) -> Self {
        Self::JsError(if let Some(err) = value.dyn_ref::<js_sys::Error>() {
            String::from(err.message())
        } else if let Some(obj) = value.dyn_ref::<js_sys::Object>() {
            String::from(obj.to_string())
        } else if let Some(s) = value.dyn_ref::<js_sys::JsString>() {
            String::from(s)
        } else {
            format!("an unknown error occured: {value:?}")
        })
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self::from(&value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::JsError(e) => write!(f, "{e}"),
            Error::InvalidHandle(e) => write!(f, "{e}"),
        }
    }
}
