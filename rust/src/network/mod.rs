use godot::prelude::*;
use tokio::{io::AsyncWriteExt, net::TcpStream, runtime};

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

    fn ready(&mut self) {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime!");

        rt.spawn(async {
            match TcpStream::connect("127.0.0.1:23541").await {
                Ok(mut stream) => match stream.write_all(b"Hello, world!").await {
                    Ok(_) => godot_print!("Attempted to write message to TCP stream..."),
                    Err(e) => panic!("{e:?}"),
                },
                Err(e) => panic!("{e:?}"),
            }
        });
    }
}
