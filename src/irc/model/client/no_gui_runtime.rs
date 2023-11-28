use std::{
    collections::VecDeque,
    io::{stdin, BufRead, BufReader, Stdin},
    thread::{self, JoinHandle},
};

use crate::irc::{
    constants::ERR_SERVERERR,
    message::{
        generic_message::GenericMessage, nickname::Nickname, password::Password, user::User,
        Command, FromGeneric, MessageError, Serializable,
    },
    model::ServerError,
};

use super::Client;

impl Client {
    pub fn client_run(mut self, stream: &mut Stdin) -> Result<(), ServerError> {
        let (pass, nick, user) = self.get_credentials();
        let _ = self.write_to_sv(&pass);
        let _ = self.write_to_sv(&nick);
        let _ = self.write_to_sv(&user);
        let thread = self.client_listen()?;
        let reader = BufReader::new(stream);
        for line in reader.lines().flatten() {
            if self.write_to_sv(&line).is_err() {
                break;
            }
        }

        self.tcp_destroy()?;

        self.thread_destroy(thread)?;

        Ok(())
    }

    fn client_listen(&self) -> Result<JoinHandle<()>, ServerError> {
        let err = ServerError {
            code: ERR_SERVERERR,
            msg: "Data client can't listen".to_string(),
        };
        let stream = match self.stream.as_ref().ok_or(err)?.try_clone() {
            Ok(s) => s,
            Err(_) => {
                self.tcp_destroy()?;
                return Err(ServerError {
                    code: ERR_SERVERERR,
                    msg: "Can't create enough TCP streams to work properly".to_string(),
                });
            }
        };

        let t = thread::spawn(move || {
            let mut lines = BufReader::new(stream).lines();
            loop {
                if let Some(line) = lines.next() {
                    match line {
                        Ok(p) => println!("{}", p),
                        Err(_) => {
                            println!("Disconnected from the server. Terminating ");
                            break;
                        }
                    }
                }
            }
        });
        Ok(t)
    }
}

impl Client {
    fn get_password(&self) -> Result<String, MessageError> {
        let mut pass = String::new();
        println!("Please, enter password:");
        while pass.is_empty() {
            match stdin().read_line(&mut pass) {
                Ok(_) => {}
                Err(_) => {
                    println!("Please, try again");
                    pass.clear();
                }
            }
        }
        let pass = pass.trim_end().as_bytes();
        Ok(Password::from_generic(GenericMessage {
            command: Command::Password,
            prefix: None,
            parameters: VecDeque::from(vec![pass]),
        })?
        .serialize())
    }

    fn get_nick(&self) -> Result<String, MessageError> {
        let mut nick = String::new();
        println!("Please, enter nick:");
        while nick.is_empty() {
            match stdin().read_line(&mut nick) {
                Ok(_) => {}
                Err(_) => {
                    println!("Please, try again");
                    nick.clear();
                }
            }
        }
        let nick = nick.trim_end().as_bytes();
        Ok(Nickname::from_generic(GenericMessage {
            command: Command::Nick,
            prefix: None,
            parameters: VecDeque::from(vec![nick]),
        })?
        .serialize())
    }

    fn get_username(&self) -> Result<String, MessageError> {
        let mut username = String::new();
        println!("Please, enter username:");
        while username.is_empty() {
            match stdin().read_line(&mut username) {
                Ok(_) => {}
                Err(_) => {
                    println!("Please, try again");
                    username.clear();
                }
            }
        }
        let username = username.trim_end().as_bytes();
        Ok(User::from_generic(GenericMessage {
            command: Command::User,
            prefix: None,
            parameters: VecDeque::from(vec![username, b"irc.fi.uba", b"server01", b":carolina"]),
        })?
        .serialize())
    }

    fn get_credentials(&self) -> (String, String, String) {
        let mut pass = String::new();
        while pass.is_empty() {
            pass = match self.get_password() {
                Ok(v) => v,
                Err(_) => {
                    pass.clear();
                    pass
                }
            }
        }
        let mut nick = String::new();
        while nick.is_empty() {
            nick = match self.get_nick() {
                Ok(v) => v,
                Err(_) => {
                    nick.clear();
                    nick
                }
            }
        }
        let mut user = String::new();
        while user.is_empty() {
            user = match self.get_username() {
                Ok(v) => v,
                Err(_) => {
                    user.clear();
                    user
                }
            }
        }
        (pass, nick, user)
    }
}
