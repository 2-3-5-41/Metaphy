use godot::prelude::*;
use metaphy_network::{Logic, Phylosopher};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{
    engine,
    runtime::{TokioRuntimeServer, TOKIO_SERVER},
};

#[derive(GodotClass)]
#[class(base=Node)]
struct NetworkPeer {
    base: Base<Node>,
    sender: Option<Sender<NetworkCommand>>,
    receiver: Option<Receiver<Logic>>,
}

#[godot_api]
impl INode for NetworkPeer {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            sender: None,
            receiver: None,
        }
    }

    fn ready(&mut self) {
        // Build our channels.
        let (node_sender, mut tokio_receiver) = channel::<NetworkCommand>(32);
        let (tokio_sender, node_receiver) = channel::<Logic>(32);

        // Get and bind the tokio runtime server singleton,
        // then get a strong reference to the runtime.
        let tokio_server = engine()
            .get_singleton(TOKIO_SERVER.into())
            .expect("There is no Tokio Runtime Server")
            .cast::<TokioRuntimeServer>();
        let rt = tokio_server.bind().runtime();

        self.sender = Some(node_sender);
        self.receiver = Some(node_receiver);

        rt.spawn(async move {
            {
                // Init debug interfaces.
                let _ = env_logger::init();
                let _ = tracing_subscriber::fmt()
                    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                    .try_init();
            }

            let mut peer =
                Phylosopher::new(None).expect("[ FETAL ] -> Failed to create Phylosopher Peer!");
            let channel = tokio_sender;

            match tokio_receiver.try_recv() {
                Ok(recv) => match recv {
                    NetworkCommand::Dial => println!("[ Recv Network Command ] -> Dial..."),
                },
                Err(err) => match err {
                    tokio::sync::mpsc::error::TryRecvError::Empty => match peer.poll().await {
                        Some(event) => match channel.send(event).await {
                            Ok(_) => (),
                            Err(err) => println!("[ Error ] -> {err:?}"),
                        },
                        None => println!("[ Info ] -> No network event to handle."),
                    },
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => panic!("[ FETAL ] -> Channel got disconnected!"),
                }
            }
        });
    }
}

#[derive(Debug)]
enum NetworkCommand {
    Dial,
}
