use js_sys::{BigInt, JsString, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

pub enum MsgToWorker {
    Init { f_ptr: usize },
}

impl MsgToWorker {
    pub fn try_to_js(self) -> Result<JsValue, JsValue> {
        let msg = Object::new();

        match self {
            MsgToWorker::Init { f_ptr } => {
                Reflect::set(&msg, &JsValue::from_str("type"), &JsValue::from_str("init"))?;
                
                /*if let Some(location) = web_sys::Document::new()?.location() {
                    let mut url = location.protocol()?;
                    url.push_str("//");
                    url.push_str(&location.host()?);

                    
                    Reflect::set(&msg, &JsValue::from_str("url"), &JsValue::from_str(&url))?;
                    
                } else {
                    // return Err(JsValue::from_str("Could not retrieve location for current document!"))
                }*/

                Reflect::set(&msg, &JsValue::from_str("module"), &wasm_bindgen::module())?;
                Reflect::set(&msg, &JsValue::from_str("memory"), &wasm_bindgen::memory())?;
                Reflect::set(&msg, &JsValue::from_str("task"), &BigInt::from(f_ptr))?;
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
                let addr = Reflect::get(&msg, &JsValue::from_str("task"))?
                    .dyn_into::<BigInt>()?;
                Ok(MsgToWorker::Init { f_ptr: u64::try_from(addr)? as usize })
            },
            _ => panic!("Message from worker had an unknown type!"),
        }
    }
}

pub enum MsgFromWorker {
    Close,
}

impl MsgFromWorker {
    pub fn try_from_js(msg: JsValue) -> Result<Self, JsValue> {
        let ty: String = Reflect::get(&msg, &JsValue::from_str("type"))?
            .dyn_into::<JsString>()?
            .into();
        match ty.as_str() {
            "close" => Ok(MsgFromWorker::Close),
            _ => panic!("Message from worker had an unknown type!"),
        }
    }
}
