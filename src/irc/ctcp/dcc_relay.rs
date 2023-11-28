use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::mpsc::Sender, usize};

use crate::gui::{
    components::irc::user_sidebar::model::nick_storage::NickStorage, GuiMessage, IncomingMessage,
    Reactor,
};

use super::{
    constants::{DCC_CHAT, DCC_CLOSE, DCC_RESUME, DCC_SEND},
    utils::{to_dcc_command, to_notice_command},
    DccMessage,
};

type ActorCreator =
    fn(usize, Sender<GuiMessage>, Rc<RefCell<NickStorage>>, &str) -> Rc<dyn DccActor>;
pub enum DccAction {
    New,
    NewMessage(String),
    FileUpdate(u64, u64),
    Destroy,
}
pub trait DccActor: std::fmt::Debug {
    fn act(&self, client: usize, action: DccAction);
}

pub struct DccRelay {
    clients_id: RefCell<HashMap<usize, String>>,
    actors: RefCell<HashMap<usize, Rc<dyn DccActor>>>,
    storage: Rc<RefCell<NickStorage>>,
    tx_uitosv: Sender<GuiMessage>,
    send_actor_creator: ActorCreator,
    chat_actor_creator: ActorCreator,
}

impl DccRelay {
    pub fn start(
        tx: Sender<GuiMessage>,
        storage: Rc<RefCell<NickStorage>>,
        send_actor: ActorCreator,
        chat_actor: ActorCreator,
    ) -> Self {
        Self {
            clients_id: RefCell::new(HashMap::new()),
            actors: RefCell::new(HashMap::new()),
            storage,
            tx_uitosv: tx,
            send_actor_creator: send_actor,
            chat_actor_creator: chat_actor,
        }
    }
}

impl DccRelay {
    fn get_ucid() -> usize {
        rand::random::<usize>()
    }
}

impl DccRelay {
    fn do_for_actor(&self, ucid: usize, action: DccAction) {
        let binding = self.actors.borrow();
        let actor = match binding.get(&ucid) {
            Some(a) => a,
            None => return,
        };

        actor.act(ucid, action);
    }
}

impl Reactor for DccRelay {
    fn react_single(&mut self, message: &IncomingMessage) {
        match message {
            IncomingMessage::Server(msg) => {
                if msg.contains(DCC_CLOSE) {
                    //TODO: PRIVMSG: DCC CLOSE
                    self.close_connection(msg);
                    return;
                }
                // Crear connection

                if let Some(nick) = self.get_nick_from_rcv_notice(msg) {
                    let ucid = DccRelay::get_ucid();
                    if let Some(actor) = self.get_actor_from_message(ucid, msg, &nick) {
                        self.insert(ucid, nick, actor.clone());
                        actor.act(ucid, DccAction::New);
                        self.incoming(ucid, msg);
                    }
                }
            }
            IncomingMessage::Client(id, msg) => {
                self.do_for_actor(*id, DccAction::NewMessage(msg.to_owned()));
            }
            IncomingMessage::Resume(id, filename, curr_size, port) => {
                let clients_ref = self.clients_id.borrow();
                let client = clients_ref.get(id);
                if let Some(nick) = client {
                    let msg = &to_notice_command(
                        nick.to_string(),
                        format!("{} {} {} {}", DCC_RESUME, filename, port, curr_size),
                    );
                    self.outgoing(*id, msg)
                }
            }
            IncomingMessage::ClientFile(id, file_size, transferred_size) => {
                self.do_for_actor(*id, DccAction::FileUpdate(*file_size, *transferred_size))
            }
        };
    }
}

impl DccRelay {
    pub fn start_new_client(&self, user_nick: String, msg: String, builder: ActorCreator) {
        let ucid = Self::get_ucid();
        let actor = builder(
            ucid,
            self.tx_uitosv.clone(),
            self.storage.clone(),
            &user_nick,
        );
        self.insert(ucid, user_nick, actor.clone());
        self.outgoing(ucid, &msg);
        actor.act(ucid, DccAction::New);
    }

    // pub fn close_client(&self, ucid: usize) {
    //     if let Some(c) = self.clients_id.borrow_mut().remove(&ucid) {
    //         self.outgoing(ucid, &to_notice_command(c, form_ctcp_cmd(DCC_CLOSE)))
    //     };
    //     if let Some(actor) = self.actors.borrow_mut().remove(&ucid) {
    //         actor.act(ucid, DccAction::Destroy)
    //     }
    // }
}

impl DccRelay {
    fn outgoing(&self, ucid: usize, msg: &str) {
        let _ = self
            .tx_uitosv
            .send(GuiMessage::OutgoingDCC(ucid, msg.to_owned()));
    }

    fn incoming(&self, ucid: usize, msg: &str) {
        let _ = self
            .tx_uitosv
            .send(GuiMessage::IncomingDCC(ucid, msg.to_owned()));
    }

    fn parse_dcc_command(&self, possible_cmd: &str) -> Option<Box<dyn DccMessage>> {
        to_dcc_command(possible_cmd)
    }

    fn get_actor_from_message(
        &self,
        ucid: usize,
        msg: &str,
        nick: &str,
    ) -> Option<Rc<dyn DccActor>> {
        self.parse_dcc_command(msg)?;
        match msg {
            _ if msg.contains(DCC_CHAT) => Some((self.chat_actor_creator)(
                ucid,
                self.tx_uitosv.clone(),
                self.storage.clone(),
                nick,
            )),
            _ if msg.contains(DCC_SEND) => Some((self.send_actor_creator)(
                ucid,
                self.tx_uitosv.clone(),
                self.storage.clone(),
                nick,
            )),
            _ if msg.contains(DCC_RESUME) => {
                if let Some(existent_ucid) = self.get_ucid_by_nick(&nick.to_string()) {
                    self.incoming(existent_ucid, msg);
                }
                None
            }
            _ => None,
        }
    }

    fn get_nick_from_rcv_notice(&self, notice: &str) -> Option<String> {
        let split: Vec<&str> = notice.split(":").collect();
        Some(split.get(0)?.to_string())
    }

    fn insert(&self, ucid: usize, nick: String, actor: Rc<dyn DccActor>) {
        self.clients_id.borrow_mut().insert(ucid, nick);
        self.actors.borrow_mut().insert(ucid, actor.clone());
    }

    fn remove(&self, ucid: usize) -> Option<Rc<dyn DccActor>> {
        self.clients_id.borrow_mut().remove(&ucid);
        self.actors.borrow_mut().remove(&ucid)
    }

    fn close_connection(&self, msg: &str) {
        if let Some(nick) = self.get_nick_from_rcv_notice(msg) {
            if let Some(ucid) = self.get_ucid_by_nick(&nick) {
                if let Some(actor) = self.remove(ucid) {
                    actor.act(ucid, DccAction::Destroy);
                }
            }
        }
    }

    fn get_ucid_by_nick(&self, nick: &String) -> Option<usize> {
        let binding = self.clients_id.borrow_mut();
        let iter = binding.iter();

        for (ucid, s_nick) in iter {
            if s_nick == nick {
                return Some(*ucid);
            }
        }

        None
    }
}
