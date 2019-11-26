use clap::{App, AppSettings, Arg, };
use zmq::{Context, Socket};
use crate::socket::{SocketParameters, create_socket};
use std::error::Error;

pub struct Chat {
    #[allow(dead_code)]
    ctx: Context,
    socket: Socket
}

impl Chat {
    pub fn new(parameters: &SocketParameters) -> Result<Self, Box<dyn Error>> {
        let ctx = zmq::Context::new();
        let socket = create_socket(&ctx, &parameters)?;
        socket.set_rcvtimeo(100)?;

        Ok(Self { ctx, socket })
    }

    pub fn send(&self, message: &str) -> Result<(), Box<dyn Error>> {
        self.socket.send(message, 0)?;
        Ok(())
    }

    pub fn send_with_id(&self, id: &str, message: &str) -> Result<(), Box<dyn Error>> {
        self.socket.send_multipart(&[id, message], 0)?;
        Ok(())
    }

    pub fn receive(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let message = self.socket.recv_multipart(0)?;
        let result = message
            .iter()
            .map(|part | {
                String::from_utf8(part.to_vec())
            }).collect::<Result<Vec<_>, _>>()?;
        Ok(result)

    }
}
pub fn chat(parameters: SocketParameters) -> Result<(), Box<dyn Error>> {
    println!("Chat {:?}", parameters.address);

    let mut chat = Chat::new(&parameters)?;
    let mut rl = rustyline::Editor::<()>::new();
    let _ = rl.load_history("history.txt");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
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
            match chat.send_with_id(&id, &message) {
                Ok(_) => println!("sent: {}", message),
                Err(err) => println!("error: {}", err)
            }
        },
    }
}



#[cfg(test)]
mod test {
    use super::*;

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



