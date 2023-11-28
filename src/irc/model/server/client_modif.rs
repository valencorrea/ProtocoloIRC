//! Modulo que se centra en las funcionalidades referentes a la modificacion de clientes por parte del server.
use std::collections::HashMap;

use crate::{
    irc::{
        constants::{
            ERR_CHANOPRIVSNEEDED, ERR_INVITEONLYCHAN, ERR_NICKNAMEINUSE, ERR_NOSUCHCHANNEL,
            ERR_NOSUCHNICK, ERR_PASSWDMISMATCH, ERR_USERONCHANNEL, INFO_PASSWORD, RPL_NICKCHANGE,
        },
        message::utils::no_such_nick,
        model::{
            client::Client, client_account::ClientAccount, connection::Connection, utils::mt,
            MTChannel, MTClient, ServerError,
        },
    },
    try_lock,
};

use super::Server;

impl Server {
    pub fn introduce_new_client(&self, client: MTClient) {
        self.push_client(client.clone());
        self.register_client(client.clone());

        let u = try_lock!(client);
        let nick_message = u.nick_message();
        let user_message = u.user_message();

        self.replicate_to_all_servers(&nick_message);
        self.replicate_to_all_servers(&user_message);
    }

    pub fn register_client(&self, client: MTClient) {
        let client = try_lock!(client);
        let mut accounts = try_lock!(self.accounts);

        accounts.remove(&client.nickname);
        let acc = mt(ClientAccount::for_client(&client));
        accounts.insert(client.nickname.to_owned(), acc);
    }

    fn re_register_client(&self, client: &Client, old_nick: &str) {
        let mut accounts = try_lock!(self.accounts);

        accounts.remove(old_nick);
        let acc = mt(ClientAccount::for_client(client));
        accounts.insert(client.nickname.to_owned(), acc);
    }

    pub fn push_client(&self, client: MTClient) {
        let nick = { try_lock!(client).nickname.to_owned() };
        let clients = &mut try_lock!(self.clients);
        clients.insert(nick, client);
    }

    fn remove_client(&self, client: MTClient) {
        let mut locked_clients = try_lock!(self.clients);
        let lclient = try_lock!(client);
        locked_clients.remove(&lclient.nickname);
    }

    pub fn is_channel_operator(&self, client: MTClient, channel_name: &str) -> bool {
        let locked_client = try_lock!(client);
        locked_client.is_channel_operator(channel_name)
    }

    fn can_change_nickname(
        &self,
        clients: &HashMap<String, MTClient>,
        client: &Client,
        new_nickname: &str,
    ) -> Result<(), ServerError> {
        let accounts = try_lock!(self.accounts);
        let nick_in_use = ServerError {
            code: ERR_NICKNAMEINUSE,
            msg: format!("{} :Nickname is already in use", new_nickname),
        };

        if !client.can_change_nick() {
            return Err(ServerError {
                code: 1001,
                msg: "Can't set nickname before setting password".to_owned(),
            });
        }
        if client.nickname == new_nickname {
            return Err(ServerError {
                code: ERR_NICKNAMEINUSE,
                msg: format!("{} :It's your current nickname", new_nickname),
            });
        }

        if clients.contains_key(new_nickname) {
            return Err(nick_in_use);
        }

        if accounts.contains_key(new_nickname) {
            return Err(nick_in_use);
        }

        Ok(())
    }

    fn change_nickname_all_channels(&self, old_nick: &str, new_nick: &str) {
        for channel in try_lock!(self.channels).values() {
            try_lock!(channel).change_nickname(old_nick, new_nick)
        }
    }

    pub fn change_nickname(
        &self,
        client: MTClient,
        new_nickname: String,
    ) -> Result<(), ServerError> {
        let mut clients = try_lock!(self.clients);
        let mut lclient = try_lock!(client);

        self.can_change_nickname(&clients, &lclient, &new_nickname)?;
        let _ = clients.remove(&lclient.nickname);

        let old_nick = lclient.set_nickname(&new_nickname);
        self.change_nickname_all_channels(&old_nick, &new_nickname);
        clients.insert(new_nickname.clone(), client.clone());
        self.re_register_client(&lclient, &old_nick);
        self.server_action_notify(&format!(
            "{}: {} {}",
            RPL_NICKCHANGE, old_nick, &new_nickname
        ));
        Ok(())
    }

    pub fn conn_can_log_in(&self, conn: &Connection, username: &str) -> bool {
        let accounts = try_lock!(self.accounts);

        let conn_password = match conn.get_password() {
            Some(p) => p.as_str(),
            None => return false,
        };
        let conn_nickname = match conn.get_nickname() {
            Some(n) => n.as_str(),
            None => return false,
        };

        // Check that the login is being done with the same nick
        if let Some(acc) = accounts.get(conn_nickname) {
            let account = try_lock!(acc);
            if let Some(account_pwd) = &account.pwd {
                // Does this server know a password for this client? If so check that the user is who they say it is.
                return account_pwd == conn_password && account.username == username;
            } else {
                // We know of this nick, but we can't know if its the correct account.
                return false;
            }
        }

        // If log in with unknown, check if username exists. If so, they are trying to login with different nick => Can't login
        for (_, acc) in accounts.iter() {
            let account = try_lock!(acc);
            if account.username == username {
                return false;
            }
        }

        // New account, can login
        true
    }

    pub fn get_client_by_nickname(&self, nickname: &str) -> Option<MTClient> {
        let clients = try_lock!(self.clients);

        clients.get(nickname).cloned()
    }

    pub fn invite_to_channel(
        &self,
        channel_name: String,
        nickname: String,
        inviter: MTClient,
    ) -> Option<ServerError> {
        let client = match self.get_client_by_nickname(&nickname) {
            Some(cl) => cl,
            None => {
                return Some(ServerError {
                    code: ERR_NOSUCHNICK,
                    msg: no_such_nick(nickname.as_bytes()),
                })
            }
        };
        let channel = match self.get_channel_by_name(&channel_name) {
            Some(ch) => ch,
            None => {
                return Some(ServerError {
                    code: ERR_NOSUCHCHANNEL,
                    msg: no_such_nick(channel_name.as_bytes()),
                })
            }
        };

        {
            let inv = try_lock!(inviter);
            if !inv.is_channel_operator(&channel_name) {
                return Some(ServerError {
                    code: ERR_CHANOPRIVSNEEDED,
                    msg: format!("{} :You're not channel operator", channel_name),
                });
            }
        }
        {
            let cl = try_lock!(client);
            if cl.is_in_channel(&channel_name) {
                return Some(ServerError {
                    code: ERR_USERONCHANNEL,
                    msg: format!("{} {}:is already on channel", &cl.nickname, channel_name),
                });
            }
        }

        try_lock!(client).add_invite(&channel_name);
        try_lock!(channel).add_client(client).expect("TODO");
        None
    }

    pub fn join_client_to_channel(
        &self,
        channel_name: &str,
        pwd: Option<String>,
        client: MTClient,
    ) -> Result<MTChannel, ServerError> {
        let ret_channel = match self.get_channel_by_name(channel_name) {
            Some(ch) => {
                {
                    let mut channel = try_lock!(ch);
                    let (is_invited, is_registered_operator) = {
                        let mut c = try_lock!(client);

                        let reg_oper = channel.registered_operators.contains_key(&c.nickname);

                        if reg_oper {
                            c.set_channel_operator(channel_name.to_owned(), ch.clone())
                        }

                        (c.is_invited(channel_name), reg_oper)
                    };
                    if is_invited || is_registered_operator {
                        channel.add_client(client.clone())?;
                    } else if channel.invite_only {
                        return Err(ServerError {
                            code: ERR_INVITEONLYCHAN,
                            msg: format!("{} :Cannot join channel", channel_name),
                        });
                    } else {
                        channel.join_client(client.clone(), pwd)?;
                    }
                }
                ch
            }
            None => {
                return Ok(self.create_channel(channel_name.to_owned(), pwd, Some(client)));
            }
        };
        let mut lclient = try_lock!(client);
        lclient.remove_invite(channel_name);
        lclient.add_channel(ret_channel.clone());
        Ok(ret_channel)
    }

    // Forcefully add the client to the channel. Only to be used in server to server communication.
    // If a server sent a JOIN command it means that it was succesfull, so I should add it without all the checks.
    pub fn add_client_to_channel(&self, client: MTClient, channel: MTChannel) {
        let channel_name = {
            let mut lchannel = try_lock!(channel);
            lchannel.insert_client(client.clone());
            lchannel.name.to_owned()
        };
        let mut lclient = try_lock!(client);
        lclient.add_channel(channel);
        lclient.remove_invite(&channel_name);
    }

    pub fn remove_client_from_channel(
        &self,
        channel_name: &str,
        client: MTClient,
    ) -> Result<(), ServerError> {
        let channel = match self.get_channel_by_name(channel_name) {
            Some(ch) => ch,
            None => {
                return Err(ServerError {
                    code: ERR_NOSUCHCHANNEL,
                    msg: format!("{} :No such channel", channel_name),
                })
            }
        };

        {
            let mut lclient = try_lock!(client);
            try_lock!(channel).remove_client_by_nickname(&lclient.nickname)?;
            lclient.remove_channel(channel_name);
        };

        if try_lock!(channel).client_amount() == 0 {
            self.remove_channel(channel_name);
        }

        Ok(())
    }

    pub fn set_irc_operator_privileges(
        &self,
        client: MTClient,
        user: String,
        pass: String,
    ) -> Result<(), ServerError> {
        let (nickname, username) = {
            let mut lclient = try_lock!(client);

            if lclient.username != user || INFO_PASSWORD != pass {
                return Err(ServerError {
                    code: ERR_PASSWDMISMATCH,
                    msg: "Password incorrect".to_owned(),
                });
            }

            lclient.server_operator = true;
            (lclient.nickname.to_owned(), lclient.username.to_owned())
        };
        self.server_broadcast(
            &format!("{}[{}] is now an IRC operator", nickname, username),
            false,
        );
        Ok(())
    }

    pub fn quit_client(&self, msg: String, client: MTClient) {
        let nick = {
            let mut lclient = try_lock!(client);

            for (_, channel) in lclient.channels.iter() {
                let _ = try_lock!(channel).remove_client_by_nickname(&lclient.nickname);
            }

            lclient.channels.clear();

            &lclient.nickname.to_owned()
        };
        self.remove_client(client);

        let message = format!("{} :{}", nick, msg);
        self.server_broadcast(&message, false);
    }

    pub fn set_client_invisible(&self, client: MTClient, to: bool) {
        let mut lclient = try_lock!(client);
        lclient.invisible = to;
    }

    pub fn set_client_receive_sv_notices(&self, client: MTClient, to: bool) {
        let mut lclient = try_lock!(client);
        lclient.rec_sv_notices = to;
    }

    pub fn set_client_sv_operator(&self, client: MTClient, to: bool) {
        if !to {
            let mut lclient = try_lock!(client);
            lclient.server_operator = false;
        }
    }

    pub fn force_set_client_sv_operator(&self, client: MTClient, to: bool) {
        let mut lclient = try_lock!(client);
        lclient.server_operator = to;
    }

    pub fn set_client_channel_operator(&self, client: MTClient, channel: MTChannel) {
        let mut lchannel = try_lock!(channel);
        let mut lclient = try_lock!(client);
        lclient.set_channel_operator(lchannel.name.to_owned(), channel.clone());
        lchannel
            .registered_operators
            .insert(lclient.nickname.to_owned(), false);
    }

    pub fn del_client_channel_operator(&self, client: MTClient, channel: MTChannel) {
        let mut lchannel = try_lock!(channel);
        let mut lclient = try_lock!(client);
        lclient.del_channel_operator(&lchannel.name);
        lchannel.registered_operators.remove(&lclient.nickname);
    }

    pub fn client_speak_in_moderated_channel(
        &self,
        client: MTClient,
        channel: MTChannel,
    ) -> Result<(), ServerError> {
        try_lock!(channel).allow_client_for_moderated(client)
    }

    pub fn client_no_speak_in_moderated_channel(
        &self,
        client: MTClient,
        channel: MTChannel,
    ) -> Result<(), ServerError> {
        try_lock!(channel).disallow_client_for_moderated(client)
    }

    pub fn add_data_client_by_nick(&self, new_nickname: String) {
        let client = mt(Client::for_data(new_nickname.clone()));

        try_lock!(self.clients).insert(new_nickname, client);
    }

    pub fn add_data_client_user_info(
        &self,
        client: MTClient,
        username: String,
        hostname: String,
        servername: String,
        realname: String,
    ) {
        {
            let mut lclient = try_lock!(client);

            lclient.username = username;
            lclient.hostname = hostname;
            lclient.servername = servername;
            lclient.realname = realname;
        }

        self.register_client(client);
    }

    pub fn set_client_away(&self, client: MTClient, away_message: String) {
        try_lock!(client).away_message = Some(away_message)
    }

    pub fn unset_client_away(&self, client: MTClient) {
        try_lock!(client).away_message = None
    }
}
