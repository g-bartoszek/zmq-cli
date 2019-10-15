use std::process::{Command, Stdio, Child, ChildStdout};
use assert_cmd::prelude::*;
use std::io::{Write, Read, BufReader, BufRead};
use std::thread::sleep;
use std::time::Duration;
use nonblock::NonBlockingReader;
use std::ops::{Deref, DerefMut};


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
    }).map(|c| Wrapper{ 0: c })
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

struct Wrapper(Child);

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

fn test_push_pull_send_listen() {
    let test_message = "TEST MESSAGE 12345";

    let mut listener = run_instance("listen --address tcp://127.0.0.1:5559 --type PULL bind").unwrap();
    let _send = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUSH connect", test_message).as_str()).unwrap();

    let mut reader  = NonBlockingReader::from_fd(listener.stdout.take().unwrap()).unwrap();
    assert!(wait_for_message(&mut reader, test_message).is_ok());
}

fn test_pub_sub() {
    let test_message_wo_topic = "TEST MESSAGE1";
    let test_message_with_topic = "TOPIC1 TEST MESSAGE2";

    let mut listener = run_instance("listen --topic TOPIC1 --address tcp://127.0.0.1:5559 --type SUB bind").unwrap();
    let mut reader  = NonBlockingReader::from_fd(listener.stdout.take().unwrap()).unwrap();

    let _send1  = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB connect",
                                 test_message_wo_topic).as_str()).unwrap();

    assert!(!wait_for_message(&mut reader, test_message_wo_topic).is_ok());

    let send2 = run_instance(format!("send --message {} --address tcp://127.0.0.1:5559 --type PUB connect",
                         test_message_with_topic).as_str()).unwrap();

    assert!(wait_for_message(&mut reader, test_message_with_topic).is_ok());
}

fn test_pair_chat() {
    let mut instance1 = run_instance("chat --address tcp://127.0.0.1:5559 --type PAIR bind").unwrap();
    let mut instance2 = run_instance("chat --address tcp://127.0.0.1:5559 --type PAIR connect").unwrap();

    let mut reader1  = NonBlockingReader::from_fd(instance1.stdout.take().unwrap()).unwrap();
    let mut reader2  = NonBlockingReader::from_fd(instance2.stdout.take().unwrap()).unwrap();


    instance1.stdin.as_mut().unwrap().write("Hi!\n".as_bytes());
    instance1.stdin.as_mut().unwrap().write("How are you?\n".as_bytes());

    sleep(Duration::from_secs(1));

    instance2.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();
    assert!(wait_for_message(&mut reader2,  "Hi!").is_ok());

    instance2.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();
    assert!(wait_for_message(&mut reader2,  "How are you?").is_ok());

    instance2.stdin.as_mut().unwrap().write("I'm fine\n".as_bytes()).unwrap();

    instance1.stdin.as_mut().unwrap().write("\n".as_bytes());
    assert!(wait_for_message(&mut reader1,  "fine").is_ok());


}

#[test]
fn integration_tests() {
    test_push_pull_send_listen();
    test_pub_sub();
    test_pair_chat();
}
