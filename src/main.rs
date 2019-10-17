use clap::{App, SubCommand, AppSettings, Arg, ArgMatches};

mod communication;

use communication::*;


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
