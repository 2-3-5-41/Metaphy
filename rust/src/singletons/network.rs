use std::{
    rc::Rc,
    sync::mpsc::{channel, Receiver, Sender},
};

use godot::{engine::Engine, prelude::*};
use metaphy_network::libp2p::{Multiaddr, PeerId};

use crate::{tasks::network::network_poll_task, TOKIO_RUNTIME_SINGLETON};

use super::runtime::TokioRuntimeSingleton;

#[derive(GodotClass)]
#[class(base = Object)]
pub struct NetworkSingleton {
    base: Base<Object>,
    net_response: Option<Rc<Receiver<NetworkResponse>>>,
    net_command: Option<Rc<Sender<NetworkCommand>>>,
}

impl NetworkSingleton {
    pub fn net_response(&self) -> Result<Rc<Receiver<NetworkResponse>>, ()> {
        match self.net_response.as_ref() {
            Some(rc) => Ok(rc.clone()),
            None => {
                godot_error!("Failed to obtain Network Response Receiver!");
                Err(())
            }
        }
    }
    pub fn net_command(&self) -> Result<Rc<Sender<NetworkCommand>>, ()> {
        match self.net_command.as_ref() {
            Some(rc) => Ok(rc.clone()),
            None => {
                godot_error!("Failed to obtain Network Command Sender!");
                Err(())
            }
        }
    }
}

#[godot_api]
impl IObject for NetworkSingleton {
    fn init(base: Base<Object>) -> Self {
        let (response_send, response_recv) = channel::<NetworkResponse>();
        let (command_send, command_recv) = channel::<NetworkCommand>();

        match Engine::singleton().get_singleton(TOKIO_RUNTIME_SINGLETON.into()) {
            Some(tokio) => {
                let mut tokio = tokio.cast::<TokioRuntimeSingleton>();
                let mut bind = tokio.bind_mut();

                match bind.get_runtime() {
                    Ok(runtime) => {
                        runtime.spawn(network_poll_task(response_send));
                    }
                    Err(_) => {
                        godot_error!("There is no tokio runtime, it wasn't correctly initialized!")
                    }
                }
            }
            None => godot_warn!("There is no tokio runtime singleton to access!"),
        }

        Self {
            base,
            net_response: Some(Rc::new(response_recv)),
            net_command: Some(Rc::new(command_send)),
        }
    }
}

pub enum NetworkResponse {
    MdnsDiscovery(Vec<(PeerId, Multiaddr)>),
}

pub enum NetworkCommand {
    Dial(Multiaddr),
}
