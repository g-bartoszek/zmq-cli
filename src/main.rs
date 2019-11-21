mod communication;
mod socket;

use clap::{App, SubCommand, AppSettings, Arg, ArgMatches};
use std::error::Error;
use communication::*;
use socket::{AssociationType, SocketType, SocketParameters};
use std::str::FromStr;
use crate::ChatCommand::Receive;
use regex::CaptureMatches;

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
        .arg(Arg::with_name("socket id")
            .long("id")
            .takes_value(true))
        .arg(Arg::with_name("bind").long("bind").conflicts_with("connect"))
        .arg(Arg::with_name("connect").long("connect"))
}

fn extract_common_parameters<'a>(matches: &'a ArgMatches) -> SocketParameters<'a> {
    let socket_type: SocketType = matches.value_of("socket type").unwrap().into();

    let a = if matches.is_present("bind") {
        AssociationType::Bind
    } else if matches.is_present("connect") {
        AssociationType::Connect
    } else {
        socket_type.default_association()
    };

    SocketParameters {
        address: matches.value_of("address").unwrap(),
        socket_type,
        association_type: a,
        socket_id: matches.value_of("socket id"),
        topic: matches.value_of("topic")
    }
}

fn main() {
    let matches = App::new("0MQ CLI")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(set_common_socket_args(SubCommand::with_name("send"),
                                           &[
                                               SocketType::PUSH.into(),
                                               SocketType::PUB.into(),
                                               SocketType::REQ.into(),
                                               SocketType::PAIR.into()])
            .arg(Arg::with_name("message")
                .long("message")
                .short("m")
                .takes_value(true)
                .required(true)
                .multiple(true)))
        .subcommand(set_common_socket_args(SubCommand::with_name("listen"),
                                           &[
                                               SocketType::PULL.into(),
                                               SocketType::SUB.into(),
                                               SocketType::PAIR.into()])
            .arg(Arg::with_name("topic")
                .long("topic")
                .short("t")
                .takes_value(true)))
        .subcommand(set_common_socket_args(SubCommand::with_name("chat"),
                                           &[
                                               SocketType::PAIR.into(),
                                               SocketType::SUB.into(),
                                               SocketType::PUB.into(),
                                               SocketType::PULL.into(),
                                               SocketType::PUSH.into(),
                                               SocketType::REQ.into(),
                                               SocketType::REP.into(),
                                               SocketType::ROUTER.into(),
                                               SocketType::DEALER.into(),
                                           ]))
        .get_matches();

    //println!("{:?}", matches);

    match match matches.subcommand() {
        ("send", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            let message = matches.values_of("message").unwrap().collect::<Vec<_>>().join(" ");
            send(parameters, &message)
        }
        ("listen", Some(matches)) => {
            let parameters = extract_common_parameters(matches);
            listen(parameters)
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

pub fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat {:?}", parameters.address);

    let mut chat = Chat::new(&parameters)?;
    let mut rl = rustyline::Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(mut line) => {
                rl.add_history_entry(line.as_str());
                execute_chat_command(&mut chat, parse_chat_command(line));
            },
            Err(rustyline::error::ReadlineError::Interrupted) | Err(rustyline::error::ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")?;
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ChatCommand {
    Receive,
    Send(String),
    SendTo(String, String)
}


fn tokenize(input: &str) -> Vec<String> {
    let mut r = Vec::<String>::new();

    for c in regex::Regex::new("['\"](.+)['\"]|([^\\s\"']\\S*[^\\s\"'])")
        .unwrap()
        .captures_iter(input) {
        //println!("Not quoted: {:?}", c);
        if let Some(m)  = c.get(1) {
            r.push(m.as_str().to_string());
        } else if let Some(m)  = c.get(2) {
            r.push(m.as_str().to_string());
        }
    }

    r
}

fn parse_chat_command(input: String) -> ChatCommand {
    let matches = App::new("chat")
        .setting(AppSettings::NoBinaryName)
        .setting(AppSettings::InferSubcommands)
        .arg(Arg::with_name("receive").long("receive").short("r").takes_value(false))
        .arg(Arg::with_name("send").long("send").short("s").takes_value(true).conflicts_with("receive"))
        .arg(Arg::with_name("receiver id").long("id").takes_value(true).conflicts_with("receive"))
        .get_matches_from_safe(tokenize(input.as_str()).into_iter());

    if let Ok(m) = matches {
        if m.is_present("receive") {
            return ChatCommand::Receive;
        } else if m.is_present("send") {
            if m.is_present("receiver id") {
                return ChatCommand::SendTo(m.value_of("receiver id").unwrap().to_string(),
                                           m.values_of("send").unwrap().map(str::to_string).collect());
            }
            return ChatCommand::Send(m.values_of("send").unwrap().map(str::to_string).collect());
        }
    }

    if input.is_empty() {
        return ChatCommand::Receive;
    }

    ChatCommand::Send(input)
}

fn execute_chat_command(chat: &mut Chat, command: ChatCommand) {
    match command {
        ChatCommand::Receive => {
            if let Ok(message) = chat.receive() {
                println!("received: {:?}", message);
            }
        },
        ChatCommand::Send(message) => {
            match chat.send(&message) {
                Ok(_) => println!("sent: {}", message),
                Err(err) => println!("error: {}", err)
            }
        },
        ChatCommand::SendTo(id, message) => {
            match chat.send(&message) {
                Ok(_) => println!("sent: {}", message),
                Err(err) => println!("error: {}", err)
            }
        },
    }
}



#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn tokenizing() {
        assert_eq!(vec!["word", "word2"], tokenize("word word2"));
        assert_eq!(vec!["two words"], tokenize("\"two words\""));
        assert_eq!(vec!["word", "two words"], tokenize("word \"two words\""));
        assert_eq!(vec!["word", "two words"], tokenize("word 'two words'"));
    }

    #[test]
    fn chat_command_parsing() {
        assert_eq!(ChatCommand::Receive, parse_chat_command("".to_string()));
        assert_eq!(ChatCommand::Receive, parse_chat_command("--receive".to_string()));
        assert_eq!(ChatCommand::Receive, parse_chat_command("-r".to_string()));
        assert_eq!(ChatCommand::Send(String::from("message")), parse_chat_command("--send message".to_string()));
        assert_eq!(ChatCommand::Send(String::from("message")), parse_chat_command("-s message".to_string()));
        assert_eq!(ChatCommand::Send(String::from("message")), parse_chat_command("-s message".to_string()));
        assert_eq!(ChatCommand::Send(String::from("message")), parse_chat_command("message".to_string()));
        assert_eq!(ChatCommand::Send(String::from("multiple words")), parse_chat_command("multiple words".to_string()));
        assert_eq!(ChatCommand::Send(String::from("multiple words")), parse_chat_command("-s 'multiple words'".to_string()));
        assert_eq!(ChatCommand::SendTo(String::from("ID1"), String::from("message")), parse_chat_command("--id ID1 -s message".to_string()));
        assert_eq!(ChatCommand::Send(String::from("Hi again")), parse_chat_command("--send \"Hi again\"".to_string()));
    }
}



