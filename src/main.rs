use clap::{App, SubCommand, AppSettings, Arg, ArgMatches};
use std::thread::sleep;
use std::time::Duration;

fn create_socket(ctx: &zmq::Context, parameters: SocketParameters) -> zmq::Socket {
    let socket = ctx.socket(match parameters.socket_type {
        "PUB" => zmq::PUB,
        "SUB" => zmq::SUB,
        "PUSH" => zmq::PUSH,
        "PULL" => zmq::PULL,
        "PAIR" => zmq::PAIR,
        _ => zmq::PULL,
    }).unwrap();

    match parameters.association_type {
        AssociationType::Connect => socket.connect(parameters.address).unwrap(),
        AssociationType::Bind => socket.bind(parameters.address).unwrap(),
    };

    socket
}

fn listen(topic: Option<&str>, parameters: SocketParameters) {
    println!("Listening {:?}", parameters.address);
    let ctx = zmq::Context::new();

    let socket = create_socket(&ctx, parameters);

    let _ = socket.set_subscribe(topic.unwrap_or("").as_bytes());

    loop {
        let msg = socket.recv_msg(0);
        println!("message: {:?}", msg.unwrap().as_str().unwrap());
    }
}

fn send(parameters: SocketParameters, message: &str) {
    println!("Sending to {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, parameters);

    sleep(Duration::from_millis(100));
    socket.send(message, 0).unwrap();
}

fn chat(parameters: SocketParameters) {
    println!("Chat{:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, parameters);
    socket.set_rcvtimeo(1000).unwrap();
    let _ = socket.set_subscribe("".as_bytes());

    sleep(Duration::from_millis(100));

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
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

fn address_arg() -> Arg<'static, 'static> {
    Arg::with_name("address")
        .long("address")
        .short("a")
        .takes_value(true)
        .required(true)
}

fn socket_type_arg(values: &[&'static str]) -> Arg<'static, 'static> {
    Arg::with_name("socket type")
        .long("type")
        .possible_values(values)
        .default_value(values[0])
}

enum AssociationType {
    Bind,
    Connect,
}

struct SocketParameters<'a>
{
    address: &'a str,
    socket_type: &'a str,
    association_type: AssociationType,
}

fn extract_common_parameters<'a>(matches: &'a ArgMatches) -> SocketParameters<'a> {
    let socket_type = matches.value_of("socket type").unwrap();
    let a = match matches.value_of("bind or connect") {
        Some("bind") => AssociationType::Bind,
        Some("connect") => AssociationType::Connect,
        _ => match socket_type {
            "PUSH" => AssociationType::Bind,
            "PUB" => AssociationType::Bind,
            "SUB" => AssociationType::Connect,
            "PULL" => AssociationType::Connect,
            _ => AssociationType::Connect,
        }
    };

    SocketParameters {
        address: matches.value_of("address").unwrap(),
        socket_type,
        association_type: a,
    }
}

fn main() {
    let matches = App::new("0MQ CLI")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("send")
            .arg(address_arg())
            .arg(socket_type_arg(&["PUSH", "PUB", ]))
            .arg(Arg::with_name("message")
                .long("message")
                .short("m")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("bind or connect").possible_values(&["bind", "connect"])))
        .subcommand(SubCommand::with_name("listen")
            .arg(address_arg())
            .arg(socket_type_arg(&["PULL", "SUB", ]))
            .arg(Arg::with_name("bind or connect").possible_values(&["bind", "connect"]))
            .arg(Arg::with_name("topic")
                .long("topic")
                .short("t")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("chat")
            .arg(address_arg())
            .arg(socket_type_arg(&["PAIR", "PUSH", "PULL", "PUB", "SUB"]))
            .arg(Arg::with_name("bind or connect").possible_values(&["bind", "connect"])))
        .get_matches();

    //println!("{:?}", matches);

    match matches.subcommand() {
        ("send", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            let message = matches.value_of("message").unwrap();
            send(parameters, message);
        }
        ("listen", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            listen(matches.value_of("topic"), parameters);
        }
        ("chat", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            chat(parameters);
        }
        _ => {}
    }
}
