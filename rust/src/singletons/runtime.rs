use std::rc::Rc;

use godot::prelude::*;
use tokio::runtime::{self, Runtime};

#[derive(GodotClass)]
#[class(base = Object)]
pub struct TokioRuntimeSingleton {
    base: Base<Object>,
    runtime: Option<Rc<Runtime>>,
}

#[godot_api]
impl IObject for TokioRuntimeSingleton {
    fn init(base: Base<Object>) -> Self {
        let runtime = runtime::Builder::new_multi_thread().enable_all().build();

        match runtime {
            Ok(tokio) => Self {
                base,
                runtime: Some(Rc::new(tokio)),
            },
            Err(err) => {
                godot_error!("Failed to create Tokio Runtime! -> {err:?}");
                Self {
                    base,
                    runtime: None,
                }
            }
        }
    }
}

#[godot_api]
impl TokioRuntimeSingleton {}

impl TokioRuntimeSingleton {
    pub fn get_runtime(&mut self) -> Result<Rc<Runtime>, ()> {
        if let Some(runtime_ref) = self.runtime.as_mut() {
            Ok(runtime_ref.clone())
        } else {
            Err(())
        }
    }
}
