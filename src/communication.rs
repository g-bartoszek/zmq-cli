
use std::time::Duration;
use std::error::Error;
use std::thread::sleep;
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
        socket.send(topic, zmq::SNDMORE)?
    }

    socket.send(message, 0)?;
    Ok(())
}


