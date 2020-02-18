use std::error::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum AssociationType {
    #[serde(alias = "bind")]
    Bind,
    #[serde(alias = "connect")]
    Connect,
}

impl Default for AssociationType {
    fn default() -> Self {
        AssociationType::Bind
    }
}

#[derive(Default, Deserialize)]
pub struct SocketParameters<'a>
{
    pub address: &'a str,
    pub socket_type: SocketType,
    pub association_type: AssociationType,
    pub socket_id: Option<&'a str>,
    pub topic: Option<&'a str>,
}

#[derive(Deserialize)]
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

impl Default for SocketType {
    fn default() -> Self {
        SocketType::PAIR
    }
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

pub fn parse(json: &str) -> SocketParameters {
    serde_json::from_str(json).unwrap() //.unwrap_or_else(|_| SocketParameters::default())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_socket_parameters_from_json() {
        let json =
            r#"{
                "address": "tcp://localhost:5559",
                "socket_type": "PULL",
                "association_type": "bind",
                "socket_id": "ID1",
                "topic": "TOPIC1"
            }"#;

        let parsed = parse(json);
        assert_eq!(Some("TOPIC1"), parsed.topic);
    }
}

