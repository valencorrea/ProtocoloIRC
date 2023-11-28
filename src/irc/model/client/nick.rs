use super::Client;

impl Client {
    pub fn can_change_nick(&self) -> bool {
        true
    }

    pub fn set_nickname(&mut self, new_nick: &str) -> String {
        let old_nickname = self.nickname.to_owned();
        self.nickname = new_nick.to_owned();
        old_nickname
    }
}
