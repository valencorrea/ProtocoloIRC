//! Modulo que se centra en las funcionalidades referentes a la modificacion de canales por parte del server.
use crate::{
    irc::{
        constants::{ERR_CANNOTSENDTOCHAN, ERR_NOSUCHCHANNEL, ERR_NOSUCHNICK, RPL_AWAY},
        message::utils::no_such_nick,
        model::{client::Client, MTChannel, MTClient, ServerError},
    },
    try_lock,
};

use super::Server;

impl Server {
    pub fn send_message_to_local_client(&self, client: &mut Client, message: &str) {
        if client.write_to_sv(message).is_err() {
            println!("[ERROR] Can't send message to client");
            println!("[ERROR] {:?}", client);
        };
    }

    fn send_message_to_client(
        &self,
        client: &mut Client,
        client_message: &str,
        server_message: &str,
        auto_replicate: bool,
    ) -> Result<(), ServerError> {
        if let Some(away_message) = &client.away_message {
            return Err(ServerError {
                code: RPL_AWAY,
                msg: format!("{} :{}", client.nickname, away_message),
            });
        }
        if client.servername != self.host && auto_replicate {
            self.replicate_to_servername(server_message, &client.servername, None)
        } else {
            self.send_message_to_local_client(client, client_message);
            Ok(())
        }
    }

    pub fn send_messsage_to_channel(
        &self,
        channel: MTChannel,
        client_message: &str,
        server_message: &str,
        auto_replicate: bool,
    ) {
        for (_, c) in try_lock!(channel).clients.iter() {
            self.send_message_to_local_client(&mut *try_lock!(c), client_message);
        }
        if auto_replicate {
            self.replicate_to_all_servers(server_message);
        }
    }

    pub fn server_broadcast(&self, msg: &str, info: bool) {
        let lclients = try_lock!(self.clients);
        for (_, c) in lclients.iter() {
            let mut locked_c = try_lock!(c);
            if !info && locked_c.rec_sv_notices {
                self.send_message_to_local_client(&mut locked_c, msg);
            }
        }
    }

    pub fn server_action_notify(&self, msg: &str) {
        let lclients = try_lock!(self.clients);
        for (_, c) in lclients.iter() {
            let mut locked_c = try_lock!(c);
            self.send_message_to_local_client(&mut locked_c, msg);
        }
    }

    pub fn try_send_message_to_client(
        &self,
        nickname: &str,
        client_message: &str,
        server_message: &str,
        auto_replicate: bool,
    ) -> Result<(), ServerError> {
        let clients = try_lock!(self.clients);
        match clients.get(nickname) {
            Some(cl) => {
                let mut client = try_lock!(cl);
                self.send_message_to_client(
                    &mut client,
                    client_message,
                    server_message,
                    auto_replicate,
                )?;
                Ok(())
            }
            None => Err(ServerError {
                code: ERR_NOSUCHNICK,
                msg: no_such_nick(nickname.as_bytes()),
            }),
        }
    }

    pub fn try_send_message_to_channel(
        &self,
        client: MTClient,
        channel_name: &str,
        client_message: &str,
        server_message: &str,
        auto_replicate: bool,
    ) -> Result<(), ServerError> {
        let channel = match self.get_channel_by_name(channel_name) {
            Some(v) => v,
            None => {
                return Err(ServerError {
                    code: ERR_NOSUCHCHANNEL,
                    msg: format!("{} :No such channel", channel_name),
                })
            }
        };
        self.can_send_to_channel(&client, &channel, channel_name)?;

        self.send_messsage_to_channel(channel, client_message, server_message, auto_replicate);
        Ok(())
    }

    pub fn can_send_to_channel(
        &self,
        client: &MTClient,
        channel: &MTChannel,
        channel_name: &str,
    ) -> Result<(), ServerError> {
        let err = ServerError {
            code: ERR_CANNOTSENDTOCHAN,
            msg: format!("{} :Cannot send to channel", channel_name),
        };
        let (is_oper, belongs) = {
            let c = try_lock!(client);
            (
                c.is_channel_operator(channel_name),
                c.is_in_channel(channel_name),
            )
        };
        let ch = try_lock!(channel);
        let no_msg_outside = ch.no_msg_outside;
        let moderated = ch.moderated;
        let allowed_to = ch.is_allowed_for_moderated(client) || is_oper;

        if no_msg_outside && !belongs {
            return Err(err);
        }

        if moderated && !allowed_to {
            return Err(err);
        }

        Ok(())
    }
}
