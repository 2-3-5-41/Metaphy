use godot::{engine::Engine, prelude::*};

mod network;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    // fn on_level_init(level: InitLevel) {
    //     let mut engine = Engine::singleton();
    //     match level {
    //         InitLevel::Scene => (),
    //         _ => (),
    //     }
    // }

    // fn on_level_deinit(level: InitLevel) {
    //     let mut engine = Engine::singleton();

    //     match level {
    //         InitLevel::Scene => (),
    //         _ => (),
    //     }
    // }
}
