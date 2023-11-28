//! Modulo que se centra en las funcionalidades referentes a la modificacion por parte del server.
use std::{
    collections::HashMap,
    sync::mpsc::Receiver,
    thread::{JoinHandle, ThreadId},
};

use super::{ThreadManagement, ThreadMap};

fn clean_thread(tid: ThreadId, handle: JoinHandle<()>) {
    match handle.join() {
        Ok(_) => println!("[SERVER - THREAD MANAGEMENT]: Cleaning thread {:?}", tid),
        Err(e) => println!(
            "[SERVER - THREAD MANAGEMENT]: Couldn't clean thread {:?}, {:?}",
            tid, e
        ),
    };
}

pub fn thread_manager(rx: Receiver<ThreadManagement>) {
    let mut thread_map: ThreadMap = HashMap::new();
    while let Ok(id_rcv) = rx.recv() {
        match id_rcv {
            ThreadManagement::KeepTrack(jh) => {
                let t_id = jh.thread().id();
                thread_map.insert(t_id, jh);
            }
            ThreadManagement::Clean(tid) => {
                if let Some(handle) = thread_map.remove(&tid) {
                    clean_thread(tid, handle)
                }
            }
            ThreadManagement::KillAll => {
                break;
            }
        }
    }
}
