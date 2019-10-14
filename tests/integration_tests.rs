use std::process::{Command, Stdio};
use assert_cmd::prelude::*;
use std::time::Duration;


#[test]
fn test_push_pull_send_listen() {
    let test_message = "TEST MESSAGE 122345";

    let mut listener = Command::cargo_bin("rzmq").unwrap()
        .args(&["listen", "--address", "tcp://127.0.0.1:5559", "--type", "PULL", "bind"])
        .stdout(Stdio::piped()).spawn().unwrap();

    let mut send = Command::cargo_bin("rzmq").unwrap()
        .args(&["send", "--message", test_message, "--address", "tcp://127.0.0.1:5559", "--type", "PUSH", "connect"])
        .spawn().unwrap();

    std::thread::sleep(Duration::from_millis(200));

    send.kill().unwrap();
    listener.kill().unwrap();

    let output = std::str::from_utf8(listener.wait_with_output().unwrap().stdout.as_slice()).unwrap().to_string();

    println!("OUTPUT: {:?}", output);

    assert!(output.contains(test_message));
}