use crate::{
    singletons::network::{NetworkCommand, NetworkResponse, NetworkSingleton},
    METAPHY_NETWORK_SINGLETON,
};
use godot::{
    engine::{
        control::LayoutPreset, Button, Engine, HBoxContainer, IButton, IPanel, Label, Panel,
        VBoxContainer,
    },
    prelude::*,
};
use metaphy_network::libp2p::Multiaddr;
use std::{rc::Rc, sync::mpsc::Receiver};

#[derive(GodotClass)]
#[class(base = Node)]
pub struct NetworkSync {
    base: Base<Node>,
    responses: Rc<Receiver<NetworkResponse>>,

    #[export]
    content: Option<Gd<VBoxContainer>>,
}

#[godot_api]
impl INode for NetworkSync {
    fn init(base: Base<Node>) -> Self {
        let network = Engine::singleton()
            .get_singleton(METAPHY_NETWORK_SINGLETON.into())
            .expect("There is no network singleton!")
            .cast::<NetworkSingleton>();
        let bind = network.bind();
        Self {
            base,
            responses: bind
                .net_response()
                .expect("There is no response channel on the network!"),
            content: None,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        match self.responses.try_recv() {
            Ok(resp) => match resp {
                NetworkResponse::MdnsDiscovery(discovery) => {
                    discovery.into_iter().for_each(|(_id, addr)| {
                        let mut instance = PeerDialer::new_alloc();

                        // Bind and set data in an isolated scope.
                        {
                            let mut bind = instance.bind_mut();
                            bind.set_dial_addr(addr);
                        }

                        match self.content.as_mut() {
                            Some(render) => render.add_child(instance.upcast::<Node>()),
                            None => godot_error!("Cannot add `PeerDialer` instance to content."),
                        }
                    })
                }
            },
            Err(err) => match err {
                std::sync::mpsc::TryRecvError::Empty => (),
                std::sync::mpsc::TryRecvError::Disconnected => panic!("Network Responses channel has been disconnected!"),
            },
        };
    }
}

#[godot_api]
impl NetworkSync {}

#[derive(GodotClass)]
#[class(base = Panel)]
pub struct PeerDialer {
    base: Base<Panel>,
    dial_addr: Option<Multiaddr>,
}

#[godot_api]
impl IPanel for PeerDialer {
    fn init(base: Base<Panel>) -> Self {
        Self {
            base,
            dial_addr: None,
        }
    }

    fn enter_tree(&mut self) {
        // Make sure it is visible in the container.
        self.base_mut()
            .set_custom_minimum_size(Vector2::new(0.0, 64.0));

        // Create `HBoxContainer` child.
        let mut h_box_container = HBoxContainer::new_alloc();
        h_box_container.set_anchors_preset(LayoutPreset::FULL_RECT);

        // Create `Label` child, and append dial address as a string.
        let mut label = Label::new_alloc();
        label.set_text(format!("{:?}", self.dial_addr.as_ref().unwrap()).into());

        // Create `DialButton` and append dial address as is to be send as a network command on click.
        let mut button = PeerDialButton::new_alloc();
        button.set_text("Dial Peer".into());
        button
            .bind_mut()
            .set_dial_addr(self.dial_addr.as_ref().unwrap().clone());

        h_box_container.add_child(label.upcast());
        h_box_container.add_child(button.upcast());
        self.base_mut().add_child(h_box_container.upcast());
    }
}

#[godot_api]
impl PeerDialer {}

impl PeerDialer {
    pub fn set_dial_addr(&mut self, addr: Multiaddr) {
        self.dial_addr = Some(addr);
    }
    pub fn get_dial_addr(&self) -> Option<&Multiaddr> {
        self.dial_addr.as_ref()
    }
}

#[derive(GodotClass)]
#[class(base = Button)]
pub struct PeerDialButton {
    base: Base<Button>,
    dial_addr: Option<Multiaddr>,
}

#[godot_api]
impl IButton for PeerDialButton {
    fn init(base: Base<Button>) -> Self {
        Self {
            base,
            dial_addr: None,
        }
    }

    fn pressed(&mut self) {
        let network = Engine::singleton()
            .get_singleton(METAPHY_NETWORK_SINGLETON.into())
            .unwrap()
            .cast::<NetworkSingleton>();
        let bind = network.bind();

        let commands = bind.net_command().unwrap();
        match commands.send(NetworkCommand::Dial(
            self.dial_addr
                .as_ref()
                .expect("There is no address to dial!")
                .clone(),
        )) {
            Ok(_) => godot_print!("Sent dial command to network!"),
            Err(err) => godot_error!("{err:?}"),
        };
    }
}

impl PeerDialButton {
    pub fn set_dial_addr(&mut self, addr: Multiaddr) {
        self.dial_addr = Some(addr)
    }
}
