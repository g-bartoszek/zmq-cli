
use std::time::Duration;
use std::error::Error;
use std::thread::sleep;
use zmq::{Context, Socket};
use crate::socket::{SocketParameters, create_socket};

pub fn listen(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Listening {:?}", parameters.address);
    let ctx = zmq::Context::new();

    let socket = create_socket(&ctx, &parameters)?;

    loop {
        let msg = socket.recv_msg(0)?;
        println!("received: {:?}", msg.as_str().unwrap());
    }
}

pub fn send(parameters: SocketParameters, message: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending to {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, &parameters)?;

    sleep(Duration::from_millis(100));

    if let Some(topic) = parameters.topic {
        socket.send(topic, SNDMORE);
    }

    socket.send(message, 0)?;
    Ok(())
}

pub struct Chat {
    ctx: Context,
    socket: Socket
}

impl Chat {
    pub fn new(parameters: &SocketParameters) -> Result<Self, Box<dyn Error>> {
        let ctx = zmq::Context::new();
        let socket = create_socket(&ctx, &parameters)?;
        socket.set_rcvtimeo(100)?;

        Ok(Self { ctx, socket })
    }

    pub fn send(&self, message: &str) -> Result<(), Box<dyn Error>> {
       self.socket.send(message, 0)?;
       Ok(())
    }

    pub fn receive(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let message = self.socket.recv_multipart(0)?;
        let result = message
            .iter()
            .map(|part | {
                String::from_utf8(part.to_vec())
            }).collect::<Result<Vec<_>, _>>()?;
        Ok(result)

    }
}

