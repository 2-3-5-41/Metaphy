use godot::prelude::*;
use metaphy_network::{init_debug_interface, Phylosopher};
use std::sync::mpsc::{Receiver, Sender};

use crate::singletons::network::{NetworkCommand, NetworkResponse};

pub async fn network_poll_task(responses: Sender<NetworkResponse>) {
    init_debug_interface();

    let mut node = Phylosopher::new(None).expect("Failed to create Phylosopher!");
    let _ = node.listen();

    // This is our network event loop.
    loop {
        match node.poll().await {
            Some(event) => match event {
                metaphy_network::Logic::Ping(ping) => godot_print!("{ping:?}"),
                metaphy_network::Logic::Protocol(identify) => match identify {
                    metaphy_network::libp2p::identify::Event::Received { peer_id, info } => {
                        godot_print!("Received Protocol ID -> {} ({:?})", peer_id, info)
                    }
                    metaphy_network::libp2p::identify::Event::Sent { peer_id } => {
                        godot_print!("Sent ID -> {peer_id}")
                    }
                    metaphy_network::libp2p::identify::Event::Pushed { peer_id, info } => {
                        godot_print!("Pushed ID -> {} ({:?})", peer_id, info)
                    }
                    metaphy_network::libp2p::identify::Event::Error { peer_id, error } => {
                        godot_error!("ID Error -> {} ({:?})", peer_id, error)
                    }
                },
                metaphy_network::Logic::Mdns(mdns) => match mdns {
                    metaphy_network::libp2p::mdns::Event::Discovered(discoveries) => {
                        match responses.send(NetworkResponse::MdnsDiscovery(discoveries)) {
                            Ok(_) => (), // Sent
                            Err(err) => godot_error!("{err:?}"),
                        }
                    }
                    metaphy_network::libp2p::mdns::Event::Expired(expirations) => {
                        expirations.into_iter().for_each(|(id, addr)| {
                            godot_print!("Mdns Expiration -> {} @ {}", id, addr)
                        });
                    }
                },
                metaphy_network::Logic::ClientRelay(relay) => match relay {
                    metaphy_network::libp2p::relay::client::Event::ReservationReqAccepted {
                        relay_peer_id,
                        renewal: _,
                        limit: _,
                    } => {
                        godot_print!("Relay Reservation Request Accepted -> {relay_peer_id}")
                    }
                    metaphy_network::libp2p::relay::client::Event::OutboundCircuitEstablished {
                        relay_peer_id,
                        limit: _,
                    } => {
                        godot_print!("Outbount Relay Circuit Established -> {relay_peer_id}")
                    }
                    metaphy_network::libp2p::relay::client::Event::InboundCircuitEstablished {
                        src_peer_id,
                        limit: _,
                    } => {
                        godot_print!("Inbound Circuit Established -> {src_peer_id}")
                    }
                },
                metaphy_network::Logic::Dcutr(dcutr) => godot_print!("{dcutr:?}"),
                metaphy_network::Logic::ClientRzv(rzv) => match rzv {
                    metaphy_network::libp2p::rendezvous::client::Event::Discovered {
                        rendezvous_node,
                        registrations,
                        cookie,
                    } => {
                        godot_print!(
                            "Rzv Discovery -> {}\nRegistrations -> {:?}\nCookie -> {:?}",
                            rendezvous_node,
                            registrations,
                            cookie
                        )
                    }
                    metaphy_network::libp2p::rendezvous::client::Event::DiscoverFailed {
                        rendezvous_node,
                        namespace,
                        error,
                    } => {
                        godot_warn!(
                            "Rzv Discovery Failed -> {} @ {:?}\nError -> {:?}",
                            rendezvous_node,
                            namespace,
                            error
                        )
                    }
                    metaphy_network::libp2p::rendezvous::client::Event::Registered {
                        rendezvous_node,
                        ttl,
                        namespace,
                    } => {
                        godot_print!(
                            "Rzv Registered -> {}\nTime to live -> {}\n Namespace -> {}",
                            rendezvous_node,
                            ttl,
                            namespace
                        )
                    }
                    metaphy_network::libp2p::rendezvous::client::Event::RegisterFailed {
                        rendezvous_node,
                        namespace,
                        error,
                    } => {
                        godot_warn!(
                            "Rzv Registration Failed -> {}\nNamespace -> {}\nError -> {:?}",
                            rendezvous_node,
                            namespace,
                            error
                        )
                    }
                    metaphy_network::libp2p::rendezvous::client::Event::Expired { peer } => {
                        godot_warn!("Rzv Expiration -> {}", peer)
                    }
                },
                metaphy_network::Logic::Kad(kad) => match kad {
                    metaphy_network::libp2p::kad::Event::InboundRequest { request } => {
                        godot_print!("Kad Inbound Request -> {request:?}")
                    }
                    metaphy_network::libp2p::kad::Event::OutboundQueryProgressed {
                        id,
                        result,
                        stats,
                        step,
                    } => {
                        godot_print!("Kad Outbound Query Progressed -> {}\nResult -> {:?}\nStats -> {:?}\nProgress -> {:?}", id, result, stats, step)
                    }
                    metaphy_network::libp2p::kad::Event::RoutingUpdated {
                        peer,
                        is_new_peer,
                        addresses,
                        bucket_range,
                        old_peer,
                    } => {
                        godot_print!("Kad Routing Update -> {}\nIs New -> {}\nAddresses -> {:?}\nBucket Range -> {:?}\nOld Peer -> {:?}", peer, is_new_peer, addresses, bucket_range, old_peer)
                    }
                    metaphy_network::libp2p::kad::Event::UnroutablePeer { peer } => {
                        godot_error!("Kad Unroutable Peer -> {peer}")
                    }
                    metaphy_network::libp2p::kad::Event::RoutablePeer { peer, address } => {
                        godot_print!("Kad Routable Peer -> {peer} @ {address:?}")
                    }
                    metaphy_network::libp2p::kad::Event::PendingRoutablePeer { peer, address } => {
                        godot_print!("Kad Pending Routable Peer -> {peer} @ {address:?}")
                    }
                    metaphy_network::libp2p::kad::Event::ModeChanged { new_mode } => {
                        godot_print!("Kad Mode Changed -> {new_mode}")
                    }
                },
            },
            None => (),
        }
    }
}
