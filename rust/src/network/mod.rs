use godot::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct TokioThreadCommunicationTest {
    base: Base<Node>,
    sender: Option<Sender<String>>,
    receiver: Option<Receiver<String>>,
}

#[godot_api]
impl INode for TokioThreadCommunicationTest {
    fn init(base: Base<Node>) -> Self {
        Self { base, sender: None, receiver: None }
    }
    fn ready(&mut self) {
        let (sa, mut ra) = channel::<String>(32);
        let (sb, rb) = channel::<String>(32);

        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        rt.spawn(async move {
            loop {
                match ra.recv().await {
                    Some(message) => {
                        println!("{message}");
                        sb.send("Pong".into()).await.unwrap();
                    },
                    None => continue,
                }
            }
        });

        self.sender = Some(sa);
        self.receiver = Some(rb)
    }
    fn physics_process(&mut self, _delat: f64) {
        let sender = self.sender.as_mut().unwrap();
        let receiver = self.receiver.as_mut().unwrap();

        match receiver.try_recv() {
            Ok(message) => {
                println!("Got -> {message}");
                match sender.try_send("Sending -> Ping".into()) {
                    Ok(_) => (),
                    Err(e) => println!("{e:?}"),
                };
            },
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Empty => match sender.try_send("Got nothing, sending -> Ping".into()) {
                    Ok(_) => (),
                    Err(e) => godot_error!("{e:?}"),
                },
                tokio::sync::mpsc::error::TryRecvError::Disconnected => godot_error!("Channels diconnected!"),
            },
        }
    }
}