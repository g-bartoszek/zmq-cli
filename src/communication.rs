use std::time::Duration;
use std::error::Error;
use std::thread::sleep;
use zmq::{Context, Socket};

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
    pub topic: Option<&'a str>,
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

    let _ = socket.set_subscribe(parameters.topic.unwrap_or("").as_bytes());

    match parameters.association_type {
        AssociationType::Connect => socket.connect(parameters.address)?,
        AssociationType::Bind => socket.bind(parameters.address)?,
    };

    Ok(socket)
}

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
    socket.send(message, 0)?;
    Ok(())
}

struct Chat {
    ctx: Context,
    socket: Socket
}

impl Chat {
    fn new(parameters: &SocketParameters) -> Result<Self, Box<dyn Error>> {
        let ctx = zmq::Context::new();
        let socket = create_socket(&ctx, &parameters)?;
        socket.set_rcvtimeo(100)?;

        Ok(Self { ctx, socket })
    }

    fn send(&self, message: &str) -> Result<(), Box<dyn Error>> {
       self.socket.send(message, 0)?;
       Ok(())
    }

    fn receive(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let message = self.socket.recv_multipart(0)?;
        let result = message
            .iter()
            .map(|part | {
                String::from_utf8(part.to_vec())
            }).collect::<Result<Vec<_>, _>>()?;
        Ok(result)

    }
}

pub fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat {:?}", parameters.address);

    let chat = Chat::new(&parameters)?;
    let mut rl = rustyline::Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(mut line) => {
                rl.add_history_entry(line.as_str());
                line.pop();
                if line.len() > 0 {
                    match chat.send(&line) {
                        Ok(_) => println!("sent: {}", line.as_str()),
                        Err(err) => println!("error: {}", err)
                    }
                } else {
                    if let Ok(message) = chat.receive() {
                        println!("received: {:?}", message);
                    }

                }
            },
            Err(rustyline::error::ReadlineError::Interrupted) | Err(rustyline::error::ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")?;
    Ok(())
}
