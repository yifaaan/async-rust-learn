pub mod executor;
pub mod reactor;
use std::{collections::HashMap, sync::atomic::AtomicUsize};

pub use executor::{Executor, Waker};
use mio::Poll;
use reactor::{event_loop, Reactor, REACTOR};
// pub use reactor::reactor;

pub fn init() -> Executor {
    start();
    Executor::new()
}

pub fn start() {
    use std::thread::spawn;

    let wakers = std::sync::Arc::new(std::sync::Mutex::new(HashMap::new()));

    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    let next_id = AtomicUsize::new(1);
    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id,
    };

    REACTOR.set(reactor).ok().expect("Reactor already running");
    // start event_loop in a new os thread
    spawn(move || event_loop(poll, wakers));
}
