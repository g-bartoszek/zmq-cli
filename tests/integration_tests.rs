use std::process::{Command, Stdio, Child, ChildStdout};
use assert_cmd::prelude::*;
use std::io::{Write};
use std::thread::sleep;
use std::time::Duration;
use nonblock::NonBlockingReader;
use std::ops::{Deref, DerefMut};

use rzmq::{chat, socket};

fn test_push_pull_send_listen() {
    let test_message = "TEST MESSAGE 12345";

    let mut listener = run_instance("listen --address tcp://127.0.0.1:5559 --type PULL --bind").unwrap();
    let _send = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUSH --connect", test_message).as_str()).unwrap();

    assert!(listener.wait_for_message(test_message).is_ok());
}

fn test_push_pull_with_json_config() {
    let test_message = "TEST MESSAGE 12345";

    let mut listener = run_instance("listen --config test_config.json").unwrap();
    let _send = run_instance(format!("send --message {} --config test_config.json --type PUSH --connect", test_message).as_str()).unwrap();

    assert!(listener.wait_for_message(test_message).is_ok());
}

fn test_pub_sub() {
    let test_message_wo_topic = "TEST MESSAGE1";
    let test_message_with_topic = "TOPIC1 TEST MESSAGE2";

    let mut listener = run_instance("listen --topic TOPIC1 --address tcp://127.0.0.1:5559 --type SUB --bind").unwrap();
    let _send1  = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB --connect",
                                 test_message_wo_topic).as_str()).unwrap();

    assert!(listener.wait_for_message( test_message_wo_topic).is_err());

    let _send2 = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB --connect",
                         test_message_with_topic).as_str()).unwrap();

    assert!(listener.wait_for_message( test_message_with_topic).is_ok());
}

fn test_pair_chat() {
    let instance1 = chat::Chat::new(&socket::SocketParameters{
        address: "tcp://127.0.0.1:5559",
        association_type: socket::AssociationType::Bind,
        socket_type: socket::SocketType::PAIR,
        socket_id: None,
        topic: None}).unwrap();
    let instance2 = chat::Chat::new(&socket::SocketParameters{
        address: "tcp://127.0.0.1:5559",
        association_type: socket::AssociationType::Connect,
        socket_type: socket::SocketType::PAIR,
        socket_id: None,
        topic: None}).unwrap();


    instance1.send("Hi!").unwrap();
    instance1.send("How are you?").unwrap();

    sleep(Duration::from_millis(500));

    assert_eq!("Hi!", instance2.receive().unwrap()[0].as_str() );
    assert_eq!("How are you?", instance2.receive().unwrap()[0].as_str());

    instance2.send("I'm fine").unwrap();
    assert_eq!("I'm fine", instance1.receive().unwrap()[0].as_str());
}

fn test_router_dealer_chat() {
    let router = chat::Chat::new(&socket::SocketParameters{
        address: "tcp://127.0.0.1:5559",
        association_type: socket::AssociationType::Bind,
        socket_type: socket::SocketType::ROUTER,
        socket_id: None,
        topic: None}).unwrap();
    let dealer = chat::Chat::new(&socket::SocketParameters{
        address: "tcp://127.0.0.1:5559",
        association_type: socket::AssociationType::Connect,
        socket_type: socket::SocketType::DEALER,
        socket_id: Some("ID1"),
        topic: None}).unwrap();
    let dealer2 = chat::Chat::new(&socket::SocketParameters{
        address: "tcp://127.0.0.1:5559",
        association_type: socket::AssociationType::Connect,
        socket_type: socket::SocketType::DEALER,
        socket_id: Some("ID2"),
        topic: None}).unwrap();


    dealer.send("MSG1").unwrap();
    dealer.send("MSG2").unwrap();

    sleep(Duration::from_millis(500));

    assert_eq!(["ID1", "MSG1"], router.receive().unwrap()[0..2]);
    assert_eq!(["ID1", "MSG2"], router.receive().unwrap()[0..2]);

    router.send_with_id("ID1", "MSG3").unwrap();
    router.send_with_id("ID2", "MSG4").unwrap();

    assert_eq!("MSG3", dealer.receive().unwrap()[0].as_str());
    assert_eq!("MSG4", dealer2.receive().unwrap()[0].as_str());

}

#[test]
fn integration_tests() {
    test_push_pull_send_listen();
    test_push_pull_with_json_config();
    test_pub_sub();
    test_pair_chat();
    test_router_dealer_chat();
}

fn run_instance(args: &str) -> Result<Wrapper, String> {
    let args = args.split_whitespace();

    Command::cargo_bin("rzmq")
        .map_err(|e| e.to_string())
        .and_then(|mut command | {
            command
                .args(args)
                .stdout(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())
        }).map(|mut c| {
        let reader = NonBlockingReader::from_fd(c.stdout.take().unwrap()).unwrap();
        Wrapper{ 0: c, 1: reader}
    })
}

struct Wrapper(Child, NonBlockingReader<ChildStdout>);

impl Drop for Wrapper {
    fn drop(&mut self) {
        self.kill().unwrap();
    }
}

impl DerefMut for Wrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Wrapper {
    type Target = Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Wrapper {
    fn wait_for_message(&mut self, message: &str) -> Result<(), &'static str> {
        let mut buffer = String::new();

        for _ in 1..10 {
            self.1.read_available_to_string(&mut buffer).unwrap();
            println!("OUT: {}", buffer);
            if buffer.contains(message) {
                println!("MATCHED: {}", buffer);
                return Ok(());
            }
            sleep(Duration::from_millis(100));
        }

        Err("Message not received")
    }

    fn _write(&mut self, input: &str) {
        self.stdin.as_mut().unwrap().write(input.as_bytes()).unwrap();
    }
}

