use clap::{App, SubCommand, AppSettings, Arg};
use std::thread::sleep;
use std::time::Duration;

fn listen(address: &str) {
    println!("Listening {:?}", address);
    let mut ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::SUB).unwrap();
    socket.set_subscribe("".as_bytes()).unwrap();
    socket.connect(address).unwrap();

    loop {
        let msg = socket.recv_msg(0);
        println!("message: {:?}", msg.unwrap().as_str().unwrap());
    }
}

fn send(address: &str, message: &str) {
    println!("Sending to {:?}", address);
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::PUB).unwrap();
    socket.connect(address).unwrap();
    sleep(Duration::from_millis(100));
    socket.send(message, 0).unwrap();
}

fn main() {
    let matches = App::new("0MQ CLI")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(SubCommand::with_name("send")
            .arg(Arg::with_name("address")
                .long("address")
                .short("a")
                .takes_value(true))
            .arg(Arg::with_name("message")
                .long("message")
                .short("m")
                .takes_value(true))
            .setting(AppSettings::ArgRequiredElseHelp))
        .subcommand(SubCommand::with_name("listen")
            .arg(Arg::with_name("address")
                .long("address")
                .short("a")
                .takes_value(true))
            .setting(AppSettings::ArgRequiredElseHelp))
        .get_matches();

    match matches.subcommand() {
        ("send", args) => {
            match args {
                Some(matches) => {
                    match (matches.value_of("address"), matches.value_of("message")) {
                        (Some(address), Some(message)) => { send(address, message); }
                        _ => {}
                    }
                }
                None => {}
            }
        },
        ("listen", args) => {
            match args {
                Some(matches) => {
                    match matches.value_of("address") {
                        Some(address) => {
                            listen(address);
                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
        _ => {}
    }
}
