use std::time::Duration;
use std::error::Error;
use std::thread::sleep;

pub enum AssociationType {
    Bind,
    Connect,
}

pub struct SocketParameters<'a>
{
    pub address: &'a str,
    pub socket_type: &'a str,
    pub association_type: AssociationType,
}

pub fn create_socket(ctx: &zmq::Context, parameters: SocketParameters) -> Result<zmq::Socket, Box<dyn Error>> {
    let socket = ctx.socket(match parameters.socket_type {
        "PUB" => zmq::PUB,
        "SUB" => zmq::SUB,
        "PUSH" => zmq::PUSH,
        "PULL" => zmq::PULL,
        "PAIR" => zmq::PAIR,
        _ => zmq::PULL,
    })?;

    match parameters.association_type {
        AssociationType::Connect => socket.connect(parameters.address)?,
        AssociationType::Bind => socket.bind(parameters.address)?,
    };

    Ok(socket)
}

pub fn listen(topic: Option<&str>, parameters: SocketParameters) -> Result<(), Box<dyn Error>>{
    println!("Listening {:?}", parameters.address);
    let ctx = zmq::Context::new();

    let socket = create_socket(&ctx, parameters)?;

    let _ = socket.set_subscribe(topic.unwrap_or("").as_bytes());

    loop {
        let msg = socket.recv_msg(0)?;
        println!("received: {:?}", msg.as_str().unwrap());
    }
}

pub fn send(parameters: SocketParameters, message: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending to {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, parameters)?;

    sleep(Duration::from_millis(100));
    socket.send(message, 0)?;
    Ok(())
}

pub fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, parameters)?;
    socket.set_rcvtimeo(1000)?;
    let _ = socket.set_subscribe("".as_bytes());

    sleep(Duration::from_millis(100));

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.pop();
        if input.len() > 0 {
            match socket.send(input.as_str(), 0) {
                Ok(_) => println!("sent: {}", input.as_str()),
                Err(err) => println!("error: {}", err)
            }
        }

        sleep(Duration::from_millis(100));

        let _ = socket.recv_msg(0).and_then(|msg| {
            println!("received: {:?}", msg.as_str().unwrap());
            Ok(())
        });
    }
}
