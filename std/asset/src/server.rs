use super::*;
use std::{
    collections::HashSet,
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    thread::{spawn, JoinHandle},
    time::Duration,
};

//  TODO: EXT Load entire directory.

#[derive(Resource)]
pub struct AssetServer {
    _fork: JoinHandle<()>,
    main: Main,
}

enum FromMainMessage {
    Load { name: String },
}

enum FromForkMessage {
    Load {
        name: String,
        data: Result<Vec<u8>, ()>,
    },
    Unload {
        name: String,
    },
}

struct Main {
    to_fork: Sender<FromMainMessage>,
    from_fork: Receiver<FromForkMessage>,
}

struct Fork<B: backend::AssetServerBackend> {
    to_main: Sender<FromForkMessage>,
    from_main: Receiver<FromMainMessage>,
    backend: B,
}

impl AssetServer {
    fn new<B: backend::AssetServerBackend>() -> Self {
        let (to_fork, from_main) = channel();
        let (to_main, from_fork) = channel();

        let fork = spawn(move || {
            let fork = Fork {
                from_main,
                to_main,
                backend: B::new(),
            };
            fork.run();
        });
        AssetServer {
            _fork: fork,
            main: Main { to_fork, from_fork },
        }
    }

    pub fn load(&self, name: String) {
        self.main
            .to_fork
            .send(FromMainMessage::Load { name })
            .unwrap();
    }

    fn take(&self) -> Option<FromForkMessage> {
        self.main.from_fork.try_recv().ok()
    }
}

pub fn asset_init<B: backend::AssetServerBackend>(g: &Galaxy) {
    g.insert_resource(AssetServer::new::<B>());
}

pub fn asset_update(g: &Galaxy) {
    if let Some(asset_server) = g.get_resource::<AssetServer>() {
        while let Some(msg) = asset_server.take() {
            match msg {
                FromForkMessage::Load { name, data } => {
                    g.insert_event(AssetLoadEvent { name, data });
                }
                FromForkMessage::Unload { name } => {
                    g.insert_event(AssetUnloadEvent { name });
                }
            }
        }
    }
}

impl<B: backend::AssetServerBackend> Fork<B> {
    fn run(mut self) {
        let mut total_loaded = HashSet::new();
        loop {
            //  Wait and Execute load requests
            let req = self.from_main.recv_timeout(Duration::from_millis(200));
            match req {
                Ok(FromMainMessage::Load { name }) => {
                    total_loaded.insert(name.clone());
                    let data = self.backend.load(&name);
                    if self
                        .to_main
                        .send(FromForkMessage::Load { name, data })
                        .is_err()
                    {
                        return;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => return,
                _ => (),
            };

            //  Check reloads
            for name in total_loaded.iter() {
                if self.backend.should_reload(&name) {
                    let data = self.backend.load(&name);
                    if self
                        .to_main
                        .send(FromForkMessage::Unload { name: name.clone() })
                        .is_err()
                        || self
                            .to_main
                            .send(FromForkMessage::Load {
                                name: name.clone(),
                                data,
                            })
                            .is_err()
                    {
                        return;
                    }
                }
            }
        }
    }
}
