use std::process::Command;
use assert_cmd::prelude::*;
use std::time::Duration;


#[test]
fn test_push_pull_send_listen() {
    let listen = std::thread::spawn(||{
        let o = Command::cargo_bin("rzmq").unwrap()
            .args(&["listen", "--address", "tcp://127.0.0.1:5559", "--type", "PULL", "bind"]).unwrap();
        println!("Output: {:?}", o);
    });

    std::thread::sleep(Duration::from_millis(100));

    let o = Command::cargo_bin("rzmq").unwrap()
        .args(&["send", "--message", "TEST1", "--address", "tcp://127.0.0.1:5559", "--type", "PUSH", "connect"]).unwrap();
    println!("Output: {:?}", o);

    listen.join();
}