use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node)]
struct NetowrkEventServer {
    base: Base<Node>,
}

#[godot_api]
impl INode for NetowrkEventServer {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}
