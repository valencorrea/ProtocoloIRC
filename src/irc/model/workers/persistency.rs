//! Modulo que se centra en las funcionalidades referentes a la persistencia.
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, LineWriter, Write},
    sync::{mpsc::Sender, Arc, Mutex},
};

use crate::irc::model::{
    channel::Channel,
    client_account::ClientAccount,
    server::Server,
    utils::{deserialize_err, mt},
};

use super::ServerCommand;

pub fn load(server: &mut Server) -> Result<(), String> {
    let sv_port = match server.host.split(':').last() {
        Some(sp) => sp.to_owned(),
        None => {
            return Err(deserialize_err("Can't read files files"));
        }
    };
    load_users(server, &sv_port)?;
    load_channels(server, &sv_port)?;
    Ok(())
}

pub fn persist(server: Arc<Server>) {
    println!("[SERVER - PERSISTENCY] Starting");
    let sv_port = match server.host.split(':').last() {
        Some(sp) => sp,
        None => {
            println!("Can't create files");
            return;
        }
    };
    let reg_users = to_csv(server.persit_registered_users());

    let channels = to_csv(server.persist_channels());

    if let Err(e) = fs::create_dir_all("./persist") {
        println!("Can't create path\n{}", e);
        return;
    };

    if let Err(e) = persist_csv(&format!("user_accounts-{}", sv_port), reg_users) {
        println!("{}", e);
        return;
    }

    if let Err(e) = persist_csv(&format!("channels-{}", sv_port), channels) {
        println!("{}", e);
        return;
    };
    println!("[SERVER - PERSISTENCY] Finished");
}

pub fn persist_notice(server: Arc<Server>, tx: Sender<ServerCommand>) {
    let _ = tx.send(ServerCommand::Persisting);
    persist(server);
    let _ = tx.send(ServerCommand::NormalOperation);
}

fn to_csv(l: Vec<Vec<String>>) -> Vec<String> {
    l.into_iter().map(|e| e.join(",")).collect()
}

fn persist_csv(filename: &str, entries: Vec<String>) -> Result<(), String> {
    let f = match File::create(format!("./persist/{}", filename)) {
        Ok(f) => f,
        Err(e) => {
            println!("{}", e);
            return Err(deserialize_err("Can't open file"));
        }
    };
    let mut fw = LineWriter::new(f);

    for entry in entries {
        match fw.write(format!("{}\r\n", entry).as_bytes()) {
            Ok(n) => {
                if n < entry.len() {
                    return Err(deserialize_err("Can't write to file all the data"));
                }
            }
            Err(_) => return Err(deserialize_err("Can't write to file")),
        };
    }

    Ok(())
}

fn load_users(server: &mut Server, postfix: &str) -> Result<(), String> {
    let user_files = File::open(format!("./persist/user_accounts-{}", postfix));
    if user_files.is_err() {
        return Err(deserialize_err("Can't open user accounts file"));
    }

    let reader = BufReader::new(user_files.unwrap());

    let mut accounts = HashMap::new();

    for l in reader.lines() {
        match l {
            Ok(line) => {
                let split = line.split(',').collect::<Vec<&str>>();

                let acc = ClientAccount::deserialize(&split)?;

                if accounts.contains_key(split[0]) {
                    return Err(deserialize_err("Duplicated nickname. Corrupted file"));
                }

                accounts.insert(acc.nickname.to_owned(), mt(acc));
            }
            Err(_) => return Err(deserialize_err("Can't read from user accounts file")),
        }
    }

    server.accounts = Mutex::new(accounts);

    Ok(())
}

fn load_channels(server: &mut Server, postfix: &str) -> Result<(), String> {
    let user_files = File::open(format!("./persist/channels-{}", postfix));
    if user_files.is_err() {
        return Err(deserialize_err("Can't open user accounts file"));
    }

    let reader = BufReader::new(user_files.unwrap());

    let mut channels = HashMap::new();

    for l in reader.lines() {
        match l {
            Ok(line) => {
                let split = line.split(',').collect::<Vec<&str>>();

                let channel = Channel::deserialize(&split)?;

                if channels.contains_key(split[0]) {
                    return Err(deserialize_err("Duplicated channel"));
                }

                channels.insert(split[0].to_owned(), mt(channel));
            }
            Err(_) => return Err(deserialize_err("Can't read from user accounts file")),
        }
    }

    server.channels = Mutex::new(channels);

    Ok(())
}
