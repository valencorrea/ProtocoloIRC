use super::Client;

impl Client {
    pub fn describe(&self) -> String {
        format!(
            "{} {} {} * : {}",
            self.nickname, self.username, self.hostname, self.realname
        )
    }

    pub fn describe_channels(&self) -> Vec<String> {
        self.channels
            .iter()
            .map(|(channel_name, _)| {
                if self.is_channel_operator(channel_name) {
                    return format!("@{}", channel_name);
                }

                channel_name.to_owned()
            })
            .collect()
    }

    pub fn nick_message(&self) -> String {
        format!("NICK {}", self.nickname.to_owned())
    }

    pub fn user_message(&self) -> String {
        format!(
            ":{} USER {} {} {} :{}",
            self.nickname, self.username, self.hostname, self.servername, self.realname
        )
    }

    pub fn rec_sv_notices_message(&self) -> String {
        if self.rec_sv_notices {
            format!("MODE {} +s", self.nickname)
        } else {
            format!("MODE {} -s", self.nickname)
        }
    }

    pub fn invisible_message(&self) -> String {
        if self.invisible {
            format!("MODE {} +i", self.nickname)
        } else {
            format!("MODE {} -i", self.nickname)
        }
    }

    pub fn sv_o_message(&self) -> String {
        if self.server_operator {
            format!("MODE {} +o", self.nickname)
        } else {
            format!("MODE {} -o", self.nickname)
        }
    }

    pub fn away_message(&self) -> Option<String> {
        if let Some(away) = &self.away_message {
            return Some(format!(":{} AWAY :{}", self.nickname, away));
        }
        None
    }

    pub fn invited_channels_messages(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        for c in self.channel_invites.iter() {
            response.push(format!("INVITE {} {}", self.nickname, c.to_owned()))
        }
        response
    }

    pub fn channels_operator_messages(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        for (c, _) in self.channel_operator.iter() {
            response.push(format!("MODE {} +o {}", c.to_owned(), self.nickname))
        }
        response
    }

    pub fn channels_messages(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        for (c, _) in self.channels.iter() {
            response.push(format!(":{} JOIN {}", self.nickname, c.to_owned()))
        }
        response
    }

    pub fn quit_message(&self) -> Option<String> {
        if self.stream.is_some() {
            return Some(format!(":{} QUIT :Server shutting down", self.nickname));
        }
        None
    }
}
