use godot::prelude::*;
use std::rc::Rc;
use tokio::runtime::{self, Runtime};

pub(crate) const TOKIO_SERVER: &'static str = "TokioRuntimeServer";

#[derive(GodotClass)]
#[class(base = Object)]
pub struct TokioRuntimeServer {
    base: Base<Object>,
    runtime: Rc<Runtime>,
}

#[godot_api]
impl IObject for TokioRuntimeServer {
    fn init(base: Base<Object>) -> Self {
        Self {
            base,
            runtime: Rc::new(
                runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("[ FETAL ] -> Failed to create Tokio Runtime!"),
            ),
        }
    }
}

#[godot_api]
impl TokioRuntimeServer {
    /// Returns a strong reference to the Tokio `Runtime`.
    pub fn runtime(&self) -> Rc<Runtime> {
        Rc::clone(&self.runtime)
    }
}
