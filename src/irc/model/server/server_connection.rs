//! Modulo que se centra en las funcionalidades referentes a la coneccion por parte del server.
use std::collections::HashMap;

use crate::{
    irc::{
        constants::{DEFAULT_SERVERNAME, ERR_ALREADYREGISTRED, ERR_NOSUCHSERVER},
        model::{MTServerConnection, ServerError},
    },
    try_lock,
};

use super::Server;

impl Server {
    fn server_messages<'a, T>(&self, server_conns: T) -> Vec<String>
    where
        T: Iterator<Item = &'a MTServerConnection>,
    {
        let mut commands = vec![];
        // Introducing myself to the new server so it has my name
        commands.push(format!("SERVER {} {} :{} Server", self.host, 1, self.host));

        // Preparing the messages for all my known servers.
        // Do it before adding the new server so it's information it's not sent.
        for serverm in server_conns {
            let server = try_lock!(serverm);
            if server.hopcount == 1 {
                commands.push(format!(
                    ":{} SERVER {} {} :{} Server",
                    self.host,
                    server.servername,
                    server.hopcount + 1,
                    server.servername
                ));
            } else if let Some(uplink) = &server.uplink {
                commands.push(format!(
                    "{} SERVER {} {} :{} Server",
                    uplink,
                    server.servername,
                    server.hopcount + 1,
                    server.servername
                ))
            }
        }

        commands
    }

    pub fn register_server_connection(
        &self,
        sv_conn: MTServerConnection,
    ) -> Result<(), ServerError> {
        let mut commands = vec![];
        {
            let sv_name = { try_lock!(sv_conn).servername.to_owned() };
            let mut servers = try_lock!(self.sv_connections);
            if servers.contains_key(&sv_name) {
                return Err(ServerError {
                    code: ERR_ALREADYREGISTRED,
                    msg: "You may not register".to_owned(),
                });
            }

            commands.append(&mut self.server_messages(servers.values()));

            servers.insert(sv_name, sv_conn.clone());
        }

        for c in try_lock!(self.channels).values() {
            let channel = try_lock!(c);
            commands.push(channel.mode_message());
            commands.push(channel.private_message());
            commands.push(channel.secret_message());
            commands.push(channel.invite_only_message());
            commands.push(channel.topic_ops_only_message());
            commands.push(channel.no_msg_outside_message());
            commands.push(channel.moderated_message());
            commands.push(channel.limit_message());
            commands.push(channel.key_message());
            if let Some(v) = channel.topic_message() {
                commands.push(v);
            }
        }
        for (_, v) in try_lock!(self.clients).iter() {
            let client = try_lock!(v);
            commands.push(client.nick_message());
            commands.push(client.user_message());
            commands.push(client.rec_sv_notices_message());
            commands.push(client.invisible_message());
            commands.push(client.sv_o_message());
            if let Some(away) = client.away_message() {
                commands.push(away);
            }
            commands.append(&mut client.invited_channels_messages());
            commands.append(&mut client.channels_operator_messages());
            commands.append(&mut client.channels_messages());
        }
        {
            let mut c = try_lock!(sv_conn);
            for com in commands {
                c.write_line(&com);
            }
        }

        Ok(())
    }

    fn should_remove(&self, currently_rem: &str, sv_conn: &MTServerConnection) -> Option<String> {
        let server = try_lock!(sv_conn);
        if let Some(uplink) = &server.uplink {
            if uplink == currently_rem {
                return Some(server.servername.to_owned());
            }
        }
        None
    }

    fn servers_to_remove(
        &self,
        svname: &str,
        sv_conns: &HashMap<String, MTServerConnection>,
    ) -> Vec<String> {
        let mut svs_to_remove = vec![];
        svs_to_remove.push(svname.to_owned());
        for serverm in sv_conns.values() {
            let rem = self.should_remove(svname, serverm);

            if let Some(to_remove) = rem {
                svs_to_remove.append(&mut self.servers_to_remove(&to_remove, sv_conns));
            }
        }

        svs_to_remove
    }

    fn remove_affected_clients(&self, svs_to_delete: &[String]) {
        let affected = {
            let clients = try_lock!(self.clients);
            let mut affected = vec![];

            for m in clients.values() {
                let is_affected = {
                    let client = try_lock!(m);
                    svs_to_delete.contains(&client.servername)
                };
                if is_affected {
                    affected.push(m.clone())
                }
            }

            affected
        };

        for client in affected {
            self.quit_client("Connection to server lost".to_owned(), client);
        }
    }

    pub fn delete_server_by_name(&self, svname: &str) {
        let mut sv_conns = try_lock!(self.sv_connections);
        let rem = self.servers_to_remove(svname, &sv_conns);

        println!("to delete: {:?}", rem);

        self.remove_affected_clients(&rem);

        for sv_name in rem {
            let _ = sv_conns.remove(&sv_name);
        }
    }

    pub fn write_to_server(&self, sv_conn: MTServerConnection, msg: &str) {
        let mut sv = try_lock!(sv_conn);

        sv.write_line(msg);
    }

    pub fn add_data_server_connection(&self, sv_conn: MTServerConnection) {
        let server_name = { try_lock!(sv_conn).servername.to_owned() };
        try_lock!(self.sv_connections).insert(server_name, sv_conn);
    }

    pub fn update_default_servername(&self, new_servername: String) {
        let mut svconns = try_lock!(self.sv_connections);
        if let Some(sv) = svconns.remove(DEFAULT_SERVERNAME) {
            {
                try_lock!(sv).servername = new_servername.to_owned();
            }
            svconns.insert(new_servername, sv);
        }
    }

    pub fn replicate_to_all_servers(&self, message: &str) {
        for svc in try_lock!(self.sv_connections).values() {
            let mut server_connection = try_lock!(svc);
            if server_connection.hopcount == 1 {
                // All servers with hopcount equals 1 are directly connected servers
                server_connection.write_line(message);
            }
        }
    }

    pub fn replicate_to_all_servers_sans_origin(&self, message: &str, origin: &str) {
        for svc in try_lock!(self.sv_connections).values() {
            let mut server_connection = try_lock!(svc);
            if server_connection.hopcount == 1 && server_connection.servername != origin {
                // All servers with hopcount equals 1 are directly connected servers
                server_connection.write_line(message);
            }
        }
    }

    fn _replicate_to_servername(
        &self,
        message: &str,
        target_server: &str,
        uplink: Option<&str>,
        sv_conns: &HashMap<String, MTServerConnection>,
    ) -> Result<(), ServerError> {
        let nosuchserver = ServerError {
            code: ERR_NOSUCHSERVER,
            msg: format!("{} :No such server", target_server),
        };

        let search = match uplink {
            Some(u) => u,
            None => target_server,
        };

        match sv_conns.get(search) {
            Some(svcm) => {
                let mut server_connection = try_lock!(svcm);
                if server_connection.hopcount == 1 {
                    server_connection.write_line(message);
                    return Ok(());
                }
                match &server_connection.uplink {
                    Some(uplink) => {
                        self.replicate_to_servername(message, target_server, Some(uplink))
                    }
                    None => Err(nosuchserver),
                }
            }
            None => Err(nosuchserver),
        }
    }

    pub fn replicate_to_servername(
        &self,
        message: &str,
        target_server: &str,
        uplink: Option<&str>,
    ) -> Result<(), ServerError> {
        let sv_conns = try_lock!(self.sv_connections);
        self._replicate_to_servername(message, target_server, uplink, &sv_conns)
    }

    pub fn introduce_server(&self, new_server: MTServerConnection) {
        let (msg, origin) = {
            let server = try_lock!(new_server);
            (
                format!(
                    ":{} SERVER {} {} :{} Server",
                    self.host, server.servername, 2, server.servername
                ),
                server.servername.to_owned(),
            )
        };
        self.replicate_to_all_servers_sans_origin(&msg, &origin);
    }
}
