use clap::{App, SubCommand, AppSettings, Arg, ArgMatches};
use std::thread::sleep;
use std::time::Duration;
use std::error::Error;

fn create_socket(ctx: &zmq::Context, parameters: SocketParameters) -> Result<zmq::Socket, Box<dyn Error>> {
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

fn listen(topic: Option<&str>, parameters: SocketParameters) -> Result<(), Box<dyn Error>>{
    println!("Listening {:?}", parameters.address);
    let ctx = zmq::Context::new();

    let socket = create_socket(&ctx, parameters)?;

    let _ = socket.set_subscribe(topic.unwrap_or("").as_bytes());

    loop {
        let msg = socket.recv_msg(0)?;
        println!("message: {:?}", msg.as_str().unwrap());
    }
}

fn send(parameters: SocketParameters, message: &str) -> Result<(), Box<dyn Error>> {
    println!("Sending to {:?}", parameters.address);
    let ctx = zmq::Context::new();
    let socket = create_socket(&ctx, parameters)?;

    sleep(Duration::from_millis(100));
    socket.send(message, 0)?;
    Ok(())
}

fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat{:?}", parameters.address);
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

fn set_common_socket_args<'a, 'b>(subcommand: App<'a, 'b>, socket_types: &[&'static str]) -> App<'a, 'b> {
    subcommand.arg(Arg::with_name("address")
        .long("address")
        .short("a")
        .takes_value(true)
        .required(true))
        .arg(Arg::with_name("socket type")
            .long("type")
            .possible_values(socket_types)
            .default_value(socket_types[0]))
        .arg(Arg::with_name("bind").long("bind").conflicts_with("connect"))
        .arg(Arg::with_name("connect").long("connect"))
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

    let a = if matches.is_present("bind") {
        AssociationType::Bind
    } else if matches.is_present("connect") {
        AssociationType::Connect
    } else {
        match socket_type {
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
        .subcommand(set_common_socket_args(SubCommand::with_name("send"),&["PUSH", "PUB", ])
            .arg(Arg::with_name("message")
                .long("message")
                .short("m")
                .takes_value(true)
                .required(true)
                .multiple(true)))
        .subcommand(set_common_socket_args(SubCommand::with_name("listen"),&["PULL", "SUB", ])
            .arg(Arg::with_name("topic")
                .long("topic")
                .short("t")
                .takes_value(true)))
        .subcommand(set_common_socket_args(SubCommand::with_name("chat"),
                                           &["PAIR", "PUSH", "PULL", "PUB", "SUB"]))
        .get_matches();

    //println!("{:?}", matches);

    match match matches.subcommand() {
        ("send", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            let message = matches.values_of("message").unwrap().collect::<Vec<_>>().join(" ");
            send(parameters, message.as_str())
        }
        ("listen", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            listen(matches.value_of("topic"), parameters)
        }
        ("chat", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            chat(parameters)
        }
        _ => Ok(())
    } {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    };
}
