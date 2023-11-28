use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use super::{IncomingMessage, Reactor};

pub struct MessageHub {
    rx: Receiver<IncomingMessage>,
    reactors: Vec<Rc<RefCell<dyn Reactor>>>,
}

impl MessageHub {
    pub fn build(rx: Receiver<IncomingMessage>) -> Self {
        Self {
            rx,
            reactors: vec![],
        }
    }

    pub fn add_reactor<T>(&mut self, reactor: T) -> Rc<RefCell<T>>
    where
        T: Reactor + 'static,
    {
        let r = Rc::new(RefCell::new(reactor));
        self.reactors.push(r.clone());
        r
    }

    pub fn listen_for_messages(&mut self, timeout: Duration) -> Result<(), RecvTimeoutError> {
        let mut incomings = vec![];

        loop {
            match self.rx.recv_timeout(timeout) {
                Ok(incoming) => incomings.push(incoming), //TODO
                Err(e) => {
                    if e != RecvTimeoutError::Timeout {
                        return Err(e);
                    }
                    break;
                }
            }
        }

        for reactor in &mut self.reactors {
            reactor.as_ref().borrow_mut().react(&incomings);
        }

        Ok(())
    }
}
