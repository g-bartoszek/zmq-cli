use clap::{App, SubCommand, AppSettings, Arg};
use std::thread::sleep;
use std::time::Duration;

fn subscribe_and_listen(topic: Option<&str>, address: &str) {
    println!("Listening {:?}", address);
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB).unwrap();

    socket.set_subscribe(topic.unwrap_or("").as_bytes()).unwrap();
    socket.connect(address).unwrap();

    loop {
        let msg = socket.recv_msg(0);
        println!("message: {:?}", msg.unwrap().as_str().unwrap());
    }
}

fn publish(address: &str, message: &str) {
    println!("Sending to {:?}", address);
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PUB).unwrap();
    socket.connect(address).unwrap();
    sleep(Duration::from_millis(100));
    socket.send(message, 0).unwrap();
}


fn pull(address: &str) {
    println!("Listening {:?}", address);
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PULL).unwrap();

    socket.connect(address).unwrap();

    loop {
        let msg = socket.recv_msg(0);
        println!("message: {:?}", msg.unwrap().as_str().unwrap());
    }
}

fn push(address: &str, message: &str) {
    println!("Sending to {:?}", address);
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PUSH).unwrap();
    socket.bind(address).unwrap();
    sleep(Duration::from_millis(100));
    socket.send(message, 0).unwrap();
}

fn address_arg() -> Arg<'static, 'static> {
    Arg::with_name("address")
        .long("address")
        .short("a")
        .takes_value(true)
        .required(true)
}

fn main() {
    let matches = App::new("0MQ CLI")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("send")
            .arg(address_arg())
            .arg(Arg::with_name("socket type")
                .long("type")
                .possible_values(&["PUB", "PUSH"])
                .default_value("PUB"))
            .arg(Arg::with_name("message")
                .long("message")
                .short("m")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("listen")
            .arg(address_arg())
            .arg(Arg::with_name("topic")
                .long("topic")
                .short("t")
                .takes_value(true))
            .arg(Arg::with_name("socket type")
                .long("type")
                .possible_values(&["SUB", "PULL"])
                .default_value("PUB")))
        .get_matches();

    println!("{:?}", matches);

    match matches.subcommand() {
        ("send", args) => {
            match args {
                Some(matches) => {
                    match (matches.value_of("address"), matches.value_of("message"), matches.value_of("socket type")) {
                        (Some(address), Some(message), Some("PUB")) => { publish(address, message); },
                        (Some(address), Some(message), Some("PUSH")) => { push(address, message); }
                        _ => {}
                    }
                }
                None => {}
            }
        }
        ("listen", args) => {
            match args {
                Some(matches) => {
                    match (matches.value_of("address"), matches.value_of("socket type")) {
                        (Some(address), Some("SUB")) => {
                            subscribe_and_listen(matches.value_of("topic"), address);
                        },
                        (Some(address), Some("PULL")) => {
                            pull(address);
                        },
                        _ => {}
                    }
                }
                None => {}
            }
        }
        _ => {}
    }
}
