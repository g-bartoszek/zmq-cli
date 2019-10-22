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
    pub socket_type: SocketType,
    pub association_type: AssociationType,
    pub socket_id: Option<&'a str>,
}

#[allow(non_camel_case_types)]
pub enum SocketType {
    PUB,
    SUB,
    REQ,
    REP,
    PUSH,
    PULL,
    PAIR,
    ROUTER,
    DEALER,
}

impl SocketType {
    pub fn default_association(&self) -> AssociationType {
        match self {
            Self::PUB => AssociationType::Bind,
            Self::SUB => AssociationType::Connect,
            Self::REQ => AssociationType::Connect,
            Self::REP => AssociationType::Bind,
            Self::PUSH => AssociationType::Connect,
            Self::PULL => AssociationType::Bind,
            Self::PAIR => AssociationType::Bind,
            Self::ROUTER => AssociationType::Bind,
            Self::DEALER => AssociationType::Bind,
        }
    }
}

impl std::convert::From<SocketType> for &str {
    fn from(s: SocketType) -> Self {
        (&s).into()
    }
}

impl std::convert::From<&SocketType> for &str {
    fn from(s: &SocketType) -> Self {
        match s {
            SocketType::PUB => "PUB",
            SocketType::SUB => "SUB",
            SocketType::REQ => "REQ",
            SocketType::REP => "REP",
            SocketType::PUSH => "PUSH",
            SocketType::PULL => "PULL",
            SocketType::PAIR => "PAIR",
            SocketType::ROUTER => "ROUTER",
            SocketType::DEALER => "DEALER",
        }
    }
}

impl std::convert::From<&str> for SocketType {
    fn from(s: &str) -> Self {
        match s {
            "PUB" => SocketType::PUB,
            "SUB" => SocketType::SUB,
            "REQ" => SocketType::REQ,
            "REP" => SocketType::REP,
            "PUSH" => SocketType::PUSH,
            "PULL" => SocketType::PULL,
            "PAIR" => SocketType::PAIR,
            "ROUTER" => SocketType::ROUTER,
            "DEALER" => SocketType::DEALER,
            _ => SocketType::PAIR,
        }
    }
}

impl std::fmt::Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

pub fn create_socket(ctx: &zmq::Context, parameters: &SocketParameters) -> Result<zmq::Socket, Box<dyn Error>> {
    println!("Socket type: {}", parameters.socket_type);

    let socket = ctx.socket(match parameters.socket_type {
        SocketType::PUB => zmq::PUB,
        SocketType::SUB => zmq::SUB,
        SocketType::PUSH => zmq::PUSH,
        SocketType::PULL => zmq::PULL,
        SocketType::PAIR => zmq::PAIR,
        SocketType::REQ => zmq::REQ,
        SocketType::REP => zmq::REP,
        SocketType::ROUTER => zmq::ROUTER,
        SocketType::DEALER => zmq::DEALER,
    })?;

    if let Some(id) = parameters.socket_id {
        socket.set_identity(id.as_bytes())?;
    }

    match parameters.association_type {
        AssociationType::Connect => socket.connect(parameters.address)?,
        AssociationType::Bind => socket.bind(parameters.address)?,
    };

    Ok(socket)
}

pub fn listen(topic: Option<&str>, parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Listening {:?}", parameters.address);
    let ctx = zmq::Context::new();

    let socket = create_socket(&ctx, &parameters)?;

    let _ = socket.set_subscribe(topic.unwrap_or("").as_bytes());

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
    socket.send(message, 0)?;
    Ok(())
}

pub fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, &parameters)?;
    socket.set_rcvtimeo(1000)?;
    let _ = socket.set_subscribe("".as_bytes());

    sleep(Duration::from_millis(100));

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.pop();
        if input.len() > 0 {
            match socket.send(&input, 0) {
                Ok(_) => println!("sent: {}", input.as_str()),
                Err(err) => println!("error: {}", err)
            }
        }

        sleep(Duration::from_millis(100));

        let _ = socket.recv_multipart(0).and_then(|msg| {
            println!("received: {:?}", msg.iter().map(|b| std::str::from_utf8(b).unwrap()).collect::<Vec<_>>());
            Ok(())
        });
    }
}
