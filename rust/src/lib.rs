use godot::{engine::Engine, prelude::*};
use runtime::{TokioRuntimeServer, TOKIO_SERVER};

mod network;
mod runtime;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Scene => engine().register_singleton(
                TOKIO_SERVER.into(),
                TokioRuntimeServer::new_alloc().upcast(),
            ),
            _ => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Scene => {
                let singleton = engine().get_singleton(TOKIO_SERVER.into()).unwrap();

                engine().unregister_singleton(TOKIO_SERVER.into());

                singleton.free()
            }
            _ => (),
        }
    }
}

pub(crate) fn engine() -> Gd<Engine> {
    Engine::singleton()
}
