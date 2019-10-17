use std::process::{Command, Stdio, Child, ChildStdout};
use assert_cmd::prelude::*;
use std::io::{Write, Read, BufReader, BufRead};
use std::thread::sleep;
use std::time::Duration;
use nonblock::NonBlockingReader;
use std::ops::{Deref, DerefMut};


fn test_push_pull_send_listen() {
    let test_message = "TEST MESSAGE 12345";

    let mut listener = run_instance("listen --address tcp://127.0.0.1:5559 --type PULL bind").unwrap();
    let _send = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUSH connect", test_message).as_str()).unwrap();

    assert!(listener.wait_for_message(test_message).is_ok());
}

fn test_pub_sub() {
    let test_message_wo_topic = "TEST MESSAGE1";
    let test_message_with_topic = "TOPIC1 TEST MESSAGE2";

    let mut listener = run_instance("listen --topic TOPIC1 --address tcp://127.0.0.1:5559 --type SUB bind").unwrap();
    let _send1  = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB connect",
                                 test_message_wo_topic).as_str()).unwrap();

    assert!(listener.wait_for_message( test_message_wo_topic).is_err());

    let send2 = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB connect",
                         test_message_with_topic).as_str()).unwrap();

    assert!(listener.wait_for_message( test_message_with_topic).is_ok());
}

fn test_pair_chat() {
    let mut instance1 = run_instance("chat --address tcp://127.0.0.1:5559 --type PAIR bind").unwrap();
    let mut instance2 = run_instance("chat --address tcp://127.0.0.1:5559 --type PAIR connect").unwrap();

    instance1.write("Hi!\n");
    instance1.write("How are you?\n");

    sleep(Duration::from_secs(1));

    instance2.write("\n");
    assert!(instance2.wait_for_message(  "Hi!").is_ok());

    instance2.write("\n");
    assert!(instance2.wait_for_message(  "How are you?").is_ok());

    instance2.write("I'm fine\n");

    instance1.write("\n");
    assert!(instance1.wait_for_message( "fine").is_ok());


}

#[test]
fn integration_tests() {
    test_push_pull_send_listen();
    test_pub_sub();
    test_pair_chat();
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

fn wait_for_message(reader: &mut NonBlockingReader<ChildStdout>, message: &str) -> Result<(), &'static str> {
    let mut buffer = String::new();

    for _ in 1..10 {
        reader.read_available_to_string(&mut buffer);
        //println!("OUT: {}", buffer);
        if buffer.contains(message) {
            //println!("MATCHED: {}", buffer);
            return Ok(());
        }
        sleep(Duration::from_millis(100));
    }

    Err("Message not received")
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
            self.1.read_available_to_string(&mut buffer);
            //println!("OUT: {}", buffer);
            if buffer.contains(message) {
                //println!("MATCHED: {}", buffer);
                return Ok(());
            }
            sleep(Duration::from_millis(100));
        }

        Err("Message not received")
    }

    fn write(&mut self, input: &str) {
        self.stdin.as_mut().unwrap().write(input.as_bytes()).unwrap();
    }
}

