use std::process::{Command, Stdio};
use assert_cmd::prelude::*;


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

#[test]
fn integration_tests() {
    test_push_pull_send_listen();
    test_pub_sub();
}
