use futures::StreamExt;
use godot::{engine::Engine, prelude::*};
use libp2p::{mdns, noise, tcp, yamux, Multiaddr};
use std::{borrow::BorrowMut, rc::Rc, time::Duration};
use tokio::{
    runtime::{self, Runtime},
    select,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
};
use tracing_subscriber::EnvFilter;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Scene => {
                Engine::singleton().register_singleton(
                    TokioRuntimeServer::SINGLETON_NAME.into(),
                    TokioRuntimeServer::new_alloc().upcast(),
                );
            }
            _ => (),
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Scene => {
                let singletons = vec![TokioRuntimeServer::SINGLETON_NAME];

                singletons.into_iter().for_each(|s| {
                    if let Some(singleton) = Engine::singleton().get_singleton(s.into()) {
                        Engine::singleton().unregister_singleton(s.into());
                        singleton.free();
                    } else {
                        godot_warn!("There is no singleton registered as -> {s}\nThere is nothing to unrigister/free...")
                    };
                });
            }
            _ => (),
        }
    }
}

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
    pub const SINGLETON_NAME: &'static str = "TokioServer";

    /// Returns a strong reference to the Tokio `Runtime`.
    pub fn runtime(&self) -> Rc<Runtime> {
        Rc::clone(&self.runtime)
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct NetTest {
    base: Base<Node>,
    channel: Option<(Sender<NetworkCommand>, Receiver<NetworkEvent>)>,
}

#[godot_api]
impl INode for NetTest {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            channel: None,
        }
    }

    fn ready(&mut self) {
        let rt = Engine::singleton()
            .get_singleton(TokioRuntimeServer::SINGLETON_NAME.into())
            .unwrap()
            .cast::<TokioRuntimeServer>()
            .bind()
            .runtime();

        let (event_send, event_recv) = channel::<NetworkEvent>(32);
        let (command_send, mut command_recv) = channel::<NetworkCommand>(32);

        self.channel = Some((command_send, event_recv));

        rt.spawn(async move {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .try_init();

            let swarm = Mutex::new(
                libp2p::SwarmBuilder::with_new_identity()
                .with_tokio()
                .with_tcp(
                    tcp::Config::default(),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .unwrap()
                .with_quic()
                .with_behaviour(|key| {
                    mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                        .unwrap()
                })
                .unwrap()
                .with_swarm_config(|conf| {
                    conf.with_idle_connection_timeout(Duration::from_secs(60))
                })
                .build()
            );

            loop {
                select! {
                    event = async {
                        swarm.lock().await.borrow_mut().select_next_some().await
                    } => match event {
                        libp2p::swarm::SwarmEvent::Behaviour(event) => match event_send.send(NetworkEvent::Mdns(event)).await {
                            Ok(_) => (),
                            Err(err) => println!("[ Network Event Sender Error ] -> {err:?}"),
                        },
                        libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, connection_id: _, endpoint: _, num_established: _, concurrent_dial_errors: _, established_in: _ } => println!("[ Libp2p | Connection Established ] -> {peer_id}"),
                        // libp2p::swarm::SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => todo!(),
                        // libp2p::swarm::SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => todo!(),
                        // libp2p::swarm::SwarmEvent::IncomingConnectionError { connection_id, local_addr, send_back_addr, error } => todo!(),
                        // libp2p::swarm::SwarmEvent::OutgoingConnectionError { connection_id, peer_id, error } => todo!(),
                        libp2p::swarm::SwarmEvent::NewListenAddr { listener_id: _, address } => println!("[ Libp2p | New listen Address ] -> {address:?}"),
                        // libp2p::swarm::SwarmEvent::ExpiredListenAddr { listener_id, address } => todo!(),
                        // libp2p::swarm::SwarmEvent::ListenerClosed { listener_id, addresses, reason } => todo!(),
                        // libp2p::swarm::SwarmEvent::ListenerError { listener_id, error } => todo!(),
                        libp2p::swarm::SwarmEvent::Dialing { peer_id, connection_id } => println!("[ Libp2p | Dialing... ] -> {peer_id:?}, {connection_id:?}"),
                        // libp2p::swarm::SwarmEvent::NewExternalAddrCandidate { address } => todo!(),
                        // libp2p::swarm::SwarmEvent::ExternalAddrConfirmed { address } => todo!(),
                        // libp2p::swarm::SwarmEvent::ExternalAddrExpired { address } => todo!(),
                        _ => println!("[ Libp2p | Unhandled Swarm Event ]"),
                    },
                    command = async {
                        // let swarm = swarm.lock().await.borrow_mut();
                        command_recv.recv().await
                    } => match command {
                        Some(command) => match command {
                            NetworkCommand::Dial(addr) => match swarm.lock().await.borrow_mut().dial(addr) {
                                Ok(_) => (),
                                Err(err) => println!("{err:?}"),
                            },
                        },
                        None => println!("No command..."),
                    }
                }
            }
        });
    }

    fn physics_process(&mut self, _delta: f64) {
        let (send, recv) = self.channel.as_mut().unwrap();
        match recv.try_recv() {
            Ok(message) => match message {
                NetworkEvent::Mdns(e) => match e {
                    mdns::Event::Discovered(discovery) => {
                        discovery.into_iter().for_each(|(_peer, addr)| {
                            match send.try_send(NetworkCommand::Dial(addr)) {
                                Ok(_) => (),
                                Err(err) => godot_error!("{err:?}"),
                            }
                        })
                    }
                    mdns::Event::Expired(_) => (),
                },
            },
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Empty => (),
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    godot_error!("Channel Disconnected!")
                }
            },
        }
    }
}

#[godot_api]
impl NetTest {}

#[derive(Debug)]
pub enum NetworkEvent {
    Mdns(mdns::Event),
}

#[derive(Debug)]
pub enum NetworkCommand {
    Dial(Multiaddr),
}
