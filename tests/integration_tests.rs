use std::process::{Command, Stdio};
use assert_cmd::prelude::*;
use std::io::{Write, Read, BufReader, BufRead};
use std::thread::sleep;
use std::time::Duration;


fn test_push_pull_send_listen() {
    let test_message = "TEST MESSAGE 122345";

    let mut listener = Command::cargo_bin("rzmq").unwrap()
        .args(&["listen", "--address", "tcp://127.0.0.1:5559", "--type", "PULL", "bind"])
        .stdout(Stdio::piped()).spawn().unwrap();

    let _ = Command::cargo_bin("rzmq").unwrap()
        .args(&["send", "--message", test_message, "--address", "tcp://127.0.0.1:5559", "--type", "PUSH", "connect"])
        .spawn().unwrap().wait();

    listener.kill().unwrap();

    let output = std::str::from_utf8(listener.wait_with_output().unwrap().stdout.as_slice()).unwrap().to_string();

    assert!(output.contains(test_message));
}

fn test_pub_sub() {
    let test_message_wo_topic = "TEST MESSAGE1";
    let test_message_with_topic = "TOPIC1 TEST MESSAGE2";

    let mut listener = Command::cargo_bin("rzmq").unwrap()
        .args(&["listen", "--topic", "TOPIC1", "--address", "tcp://127.0.0.1:5559", "--type", "SUB", "bind"])
        .stdout(Stdio::piped()).spawn().unwrap();

    let _ = Command::cargo_bin("rzmq").unwrap()
        .args(&["send", "--message", test_message_wo_topic, "--address", "tcp://127.0.0.1:5559", "--type", "PUB", "connect"])
        .spawn().unwrap().wait();

    let _ = Command::cargo_bin("rzmq").unwrap()
        .args(&["send", "--message", test_message_with_topic, "--address", "tcp://127.0.0.1:5559", "--type", "PUB", "connect"])
        .spawn().unwrap().wait();

    listener.kill().unwrap();

    let output = std::str::from_utf8(listener.wait_with_output().unwrap().stdout.as_slice()).unwrap().to_string();

    assert!(output.contains(test_message_with_topic));
    assert!(!output.contains(test_message_wo_topic));
}

fn test_pair_chat() {
    let mut instance1 = Command::cargo_bin("rzmq").unwrap()
        .args(&["chat", "--address", "tcp://127.0.0.1:5559", "--type", "PAIR", "bind"])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn().unwrap();

    let mut instance2 = Command::cargo_bin("rzmq").unwrap()
        .args(&["chat", "--address", "tcp://127.0.0.1:5559", "--type", "PAIR", "connect"])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn().unwrap();

    let mut buffer = String::new();

    sleep(Duration::from_secs(1));
    {
        let mut instance2_out = BufReader::new(instance2.stdout.as_mut().unwrap());
        let mut instance1_out = BufReader::new(instance1.stdout.as_mut().unwrap());

        instance1.stdin.as_mut().unwrap().write("Hi!\n".as_bytes());
        instance1.stdin.as_mut().unwrap().write("How are you?\n".as_bytes());

        sleep(Duration::from_secs(1));

        instance2.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();
        instance2.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();

        sleep(Duration::from_secs(1));

        instance2_out.read_line(&mut buffer);
        instance2_out.read_line(&mut buffer);
        println!("Output {}", buffer);
        assert!(buffer.contains("Hi!"));

        instance2.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();

        instance2_out.read_line(&mut buffer);
        println!("Output {}", buffer);
        assert!(buffer.contains("How are you?"));

        instance2.stdin.as_mut().unwrap().write("I'm fine\n".as_bytes()).unwrap();

        sleep(Duration::from_secs(1));

        instance1.stdin.as_mut().unwrap().write("\n".as_bytes()).unwrap();
        instance1_out.read_line(&mut buffer);
        instance1_out.read_line(&mut buffer);
        instance1_out.read_line(&mut buffer);
        instance1_out.read_line(&mut buffer);
        println!("Output {}", buffer);
        assert!(buffer.contains("fine"));
    }

    instance1.kill();
    instance2.kill();
}

#[test]
fn integration_tests() {
    test_push_pull_send_listen();
    test_pub_sub();
    test_pair_chat();
}
