use crate::runtime::Waker;
use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
};

type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

/// 只有一个实例
pub static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside an runtime context")
}

pub struct Reactor {
    pub wakers: Wakers,
    ///  to interact with the event queue in mio
    pub registry: Registry,
    pub next_id: AtomicUsize,
}

impl Reactor {
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        self.registry.register(stream, Token(id), interest).unwrap()
    }

    pub fn set_waker(&self, waker: &Waker, id: usize) {
        let _ = self
            .wakers
            .lock()
            .map(|mut w| w.insert(id, waker.clone()).is_none())
            .unwrap();
    }

    pub fn deregister(&self, stream: &mut TcpStream, id: usize) {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();
        self.registry.deregister(stream).unwrap()
    }

    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

pub fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(100);
    loop {
        poll.poll(&mut events, None).unwrap();
        for e in events.iter() {
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();
            //
            if let Some(waker) = wakers.get(&id) {
                waker.wake();
            }
        }
    }
}
