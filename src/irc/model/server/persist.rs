//! Modulo que se centra en las funcionalidades referentes a la persistencia por parte del server.
use crate::try_lock;

use super::Server;

impl Server {
    pub fn persit_registered_users(&self) -> Vec<Vec<String>> {
        let accounts = try_lock!(self.accounts);
        accounts
            .values()
            .filter_map(|acc| {
                let account = try_lock!(acc);
                account.pwd.as_ref()?;
                Some(account.serialize())
            })
            .collect()
    }

    pub fn persist_channels(&self) -> Vec<Vec<String>> {
        let channels = try_lock!(self.channels);
        channels
            .values()
            .filter_map(|channel| {
                let ch = try_lock!(channel);
                if ch.registered_operators.is_empty() {
                    return None;
                }
                Some(ch.serialize())
            })
            .collect()
    }
}
