use js_sys::{BigInt, JsString, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

use super::url::get_bindgen_url;

pub enum WorkerMessage {
    Init { f_ptr: usize },
    Close,
}

impl WorkerMessage {
    pub fn try_to_js(self) -> Result<JsValue, JsValue> {
        let msg = Object::new();

        match self {
            WorkerMessage::Init { f_ptr } => {
                Reflect::set(&msg, &JsValue::from_str("type"), &JsValue::from_str("init"))?;
                Reflect::set(
                    &msg,
                    &JsValue::from_str("url"),
                    &JsValue::from_str(&get_bindgen_url()),
                )?;
                Reflect::set(&msg, &JsValue::from_str("module"), &wasm_bindgen::module())?;
                Reflect::set(&msg, &JsValue::from_str("memory"), &wasm_bindgen::memory())?;
                Reflect::set(&msg, &JsValue::from_str("task"), &BigInt::from(f_ptr))?;
            }
            WorkerMessage::Close => {
                Reflect::set(
                    &msg,
                    &JsValue::from_str("type"),
                    &JsValue::from_str("close"),
                )?;
            }
        };

        Ok(msg.into())
    }

    pub fn try_from_js(msg: JsValue) -> Result<Self, JsValue> {
        let ty: String = Reflect::get(&msg, &JsValue::from_str("type"))?
            .dyn_into::<JsString>()?
            .into();

        match ty.as_str() {
            "init" => {
                let addr = Reflect::get(&msg, &JsValue::from_str("task"))?.dyn_into::<BigInt>()?;
                Ok(WorkerMessage::Init {
                    f_ptr: u64::try_from(addr)? as usize,
                })
            }
            "close" => Ok(WorkerMessage::Close),
            _ => panic!("Message from worker had an unknown type!"),
        }
    }
}
