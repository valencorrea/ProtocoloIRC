//! Modulo que se centra en las funcionalidades referentes a la modificacion de canales por parte del server.
use std::{
    collections::HashMap,
    io::{self, Write},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Sender},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    irc::{
        message::{
            generic_message::GenericMessage, password::Password, server::Sv, FromGeneric,
            Serializable,
        },
        model::{
            connection::Connection,
            server_connection::ServerConnection,
            utils::mt,
            workers::{
                client_management::thread_manager,
                persistency::{load, persist, persist_notice},
                server_command::listen_commands,
                ServerCommand, ThreadManagement,
            },
            ConnectionError, SERVER_ARGS, SERVER_CONNECT_ARGS, SERVER_CONNECT_IP_POS,
            SERVER_CONNECT_PASSWORD_POS, SERVER_CONNECT_PORT_POS, SERVER_PORT_POS,
        },
        responses::ResponseType,
    },
    try_lock,
};

use super::Server;

impl Server {
    pub fn create(argv: &[String]) -> Result<Server, ConnectionError> {
        if argv.len() < SERVER_ARGS {
            return Err(ConnectionError::InvalidArguments);
        }

        Ok(Server {
            host: format!("127.0.0.1:{}", &argv[SERVER_PORT_POS]),
            clients: Mutex::new(HashMap::new()),
            channels: Mutex::new(HashMap::new()),
            accounts: Mutex::new(HashMap::new()),
            sv_connections: Mutex::new(HashMap::new()),
        })
    }

    pub fn load_file(&mut self) -> Result<(), String> {
        load(self)
    }

    pub fn server_connect(
        server: Arc<Server>,
        argv: &[String],
    ) -> Result<(Sender<ResponseType>, JoinHandle<()>), ConnectionError> {
        if argv.len() != SERVER_CONNECT_ARGS {
            return Err(ConnectionError::InvalidArguments);
        }

        let pass_msg =
            match GenericMessage::parse(&format!("PASS {}\r\n", argv[SERVER_CONNECT_PASSWORD_POS]))
            {
                Ok(g) => match Password::from_generic(g) {
                    Ok(p) => format!("{}\r\n", p.serialize()),
                    Err(_) => return Err(ConnectionError::InvalidArguments),
                },
                Err(_) => return Err(ConnectionError::InvalidArguments),
            };

        let server_msg = match GenericMessage::parse(&format!(
            "SERVER {} 1 :{} Server\r\n",
            server.host, server.host
        )) {
            Ok(g) => match Sv::from_generic(g) {
                Ok(p) => format!("{}\r\n", p.serialize()),
                Err(_) => return Err(ConnectionError::InvalidArguments),
            },
            Err(_) => return Err(ConnectionError::InvalidArguments),
        };

        let mut stream = match TcpStream::connect(format!(
            "{}:{}",
            argv[SERVER_CONNECT_IP_POS], argv[SERVER_CONNECT_PORT_POS]
        )) {
            Ok(v) => v,
            Err(_) => return Err(ConnectionError::InvalidArguments),
        };

        if stream.write(pass_msg.as_bytes()).is_err() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return Err(ConnectionError::InternalServerError);
        };

        if stream.write(server_msg.as_bytes()).is_err() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return Err(ConnectionError::InternalServerError);
        };

        let (sv_connection, stream) = ServerConnection::create(
            "Unknown".to_owned(),
            argv[SERVER_CONNECT_PASSWORD_POS].to_owned(),
            stream,
        )?;

        let (tx, rx) = channel();

        let th = thread::spawn(move || {
            let _ = stream.set_nonblocking(true);
            let _ =
                Connection::handle_server_connection(server, mt(sv_connection), &stream, Some(rx));
        });

        Ok((tx, th))
    }

    pub fn server_run(server: Arc<Server>) -> std::io::Result<()> {
        let listener = TcpListener::bind(&server.host)?;
        listener
            .set_nonblocking(true)
            .expect("[SERVER] FATAL: Can't set nonblocking in TCP Listener");

        let (tx, rx) = channel();

        let (comm_tx, comm_rx) = channel();
        let commands = comm_tx.clone();

        let comm = thread::spawn(move || listen_commands(commands));

        let tmt = thread::spawn(move || thread_manager(rx));

        let (persistency, exited) = Server::launch_persistency_thread(server.clone(), comm_tx);

        let (exit, cvar) = &*exited;

        let mut curr_persisting = false;

        for stream in listener.incoming() {
            match stream {
                Ok(streamok) => {
                    Server::launch_connection_thread(server.clone(), streamok, &tx);
                }

                // Waiting for another connection, see if server tasks are to be performed
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    while let Ok(v) = comm_rx.try_recv() {
                        match v {
                            ServerCommand::Shutdown => {
                                if let Err(e) = tx.send(ThreadManagement::KillAll) {
                                    println!("[SERVER] The shutdown order couldn't be delivered\n[SERVER] {}",e);
                                    continue;
                                }
                                if curr_persisting {
                                    println!("[SERVER] Can't shutdown while persisting operation is running");
                                    continue;
                                }
                                let mut exited = try_lock!(exit);
                                *exited = true;
                                cvar.notify_one();
                            }
                            ServerCommand::Persisting => {
                                curr_persisting = true;
                            }
                            ServerCommand::NormalOperation => {
                                curr_persisting = false;
                            }
                        }
                    }
                }

                // IO error, can't do much about it
                Err(err) => {
                    println!("[SERVER] IO Error.\n[SERVER - IO ERROR]{}", err)
                }
            }

            if *(try_lock!(exit)) {
                break;
            }
        }

        println!("[SERVER] Starting shut down");
        drop(tx);
        persist(server.clone());
        // No more listening, handle all the worker threads. The only way to get here is issuing a SHUTDOWN command into the server.
        // This will preemptively destroy all client threads and connections
        server.shutdown();
        let _ = comm.join();
        let _ = tmt.join();
        let _ = persistency.join();

        println!("[SERVER] Goodbye :)");

        Ok(())
    }

    fn launch_connection_thread(
        server: Arc<Server>,
        stream: TcpStream,
        tx: &Sender<ThreadManagement>,
    ) {
        let transmiter = tx.clone();

        let thread = thread::spawn(move || {
            let _ = Connection::handle_connection(server, stream);
            match transmiter.send(ThreadManagement::Clean(thread::current().id())) {
                Ok(_) => {}
                Err(_) => {
                    println!("[SERVER-CONNECTION] Can't properly cleanup thread");
                }
            }
        });

        let _ = tx.send(ThreadManagement::KeepTrack(thread));
    }

    fn launch_persistency_thread(
        server: Arc<Server>,
        tx: Sender<ServerCommand>,
    ) -> (JoinHandle<()>, Arc<(Mutex<bool>, Condvar)>) {
        let exited = Arc::new((Mutex::new(false), Condvar::new()));
        let pair = exited.clone();
        (
            thread::spawn(move || {
                let (lock, cvar) = &*exited;

                let mut exited = try_lock!(lock);
                loop {
                    match cvar.wait_timeout(exited, Duration::from_secs(60 * 15)) {
                        Ok((v, _)) => {
                            exited = v;
                            if *exited {
                                break;
                            }
                            persist_notice(server.clone(), tx.clone());
                        }
                        Err(e) => {
                            exited = e.into_inner().0;
                        }
                    }
                }
            }),
            pair,
        )
    }

    fn shutdown_clients(&self) -> Vec<String> {
        let mut r = vec![];

        for clientm in try_lock!(self.clients).values() {
            let client = try_lock!(clientm);
            if let Some(quit) = client.quit_message() {
                r.push(quit);
            }
        }

        r
    }

    fn shutdown(&self) {
        for server_conn in try_lock!(self.sv_connections).values() {
            let mut msgs = vec![];
            msgs.append(&mut self.shutdown_clients());
            msgs.push(format!("SQUIT {} :Shutting down server", self.host));
            let mut server = try_lock!(server_conn);
            if server.hopcount == 1 {
                for msg in &msgs {
                    server.write_line(msg);
                }
            }

            server.shutdown();
        }
    }
}
