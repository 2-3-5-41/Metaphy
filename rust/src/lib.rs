use godot::{engine::Engine, prelude::*};
use singletons::{network::NetworkSingleton, runtime::TokioRuntimeSingleton};

mod nodes;
mod singletons;
mod tasks;

struct MyExtension;

pub const TOKIO_RUNTIME_SINGLETON: &str = "Tokio";
pub const METAPHY_NETWORK_SINGLETON: &str = "Metaphysical";

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        let mut engine = Engine::singleton();
        match level {
            InitLevel::Scene => {
                engine.register_singleton(
                    TOKIO_RUNTIME_SINGLETON.into(),
                    TokioRuntimeSingleton::new_alloc().upcast(),
                );
                engine.register_singleton(
                    METAPHY_NETWORK_SINGLETON.into(),
                    NetworkSingleton::new_alloc().upcast(),
                )
            }
            _ => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        let mut engine = Engine::singleton();

        match level {
            InitLevel::Scene => {
                match engine.get_singleton(TOKIO_RUNTIME_SINGLETON.into()) {
                    Some(singleton) => {
                        engine.unregister_singleton(TOKIO_RUNTIME_SINGLETON.into());
                        singleton.free();
                    }
                    None => godot_print!("No singleton `{TOKIO_RUNTIME_SINGLETON}` found..."),
                }

                match engine.get_singleton(METAPHY_NETWORK_SINGLETON.into()) {
                    Some(singleton) => {
                        engine.unregister_singleton(METAPHY_NETWORK_SINGLETON.into());
                        singleton.free();
                    }
                    None => godot_print!("No singleton `{METAPHY_NETWORK_SINGLETON}` found..."),
                }
            }
            _ => (),
        }
    }
}
