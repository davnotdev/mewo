use super::{backend::AssetServerBackend, *};
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    env::{current_dir, set_current_dir},
    fs::*,
    io,
    sync::mpsc::{channel, Receiver},
};

struct FileSystemAssetServerBackend {
    watcher_recv: Receiver<DebouncedEvent>,
    should_reload: HashMap<String, bool>,
    watcher: RecommendedWatcher,
}

const ROOT_DIR: &'static str = "app_root";

fn search_root_dir(path: &str) -> io::Result<Option<String>> {
    let dir = read_dir(path)?;
    for entry in dir {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            let raw_path = entry.path();
            let path = raw_path.to_str().unwrap();
            let split_path = path.rsplit_once("/").unwrap().1;
            if split_path == ROOT_DIR {
                return Ok(Some(canonicalize(path)?.to_str().unwrap().to_string()));
            } else {
                let res = search_root_dir(path);
                if let Ok(Some(_)) = res {
                    return res;
                }
            }
        }
    }
    Ok(None)
}

impl AssetServerBackend for FileSystemAssetServerBackend {
    fn new() -> Self
    where
        Self: Sized,
    {
        if let Some(dir) = search_root_dir(current_dir().unwrap().to_str().unwrap()).unwrap() {
            set_current_dir(dir).unwrap();
        } else {
            merr!(
                "No root. Be sure that you are not running inside `{}`.",
                ROOT_DIR
            );
            panic!("You know what you need to do by now.")
        };
        let (tx, rx) = channel();
        FileSystemAssetServerBackend {
            watcher_recv: rx,
            should_reload: HashMap::new(),
            watcher: watcher(tx, std::time::Duration::from_secs(1)).unwrap(),
        }
    }

    fn load(&mut self, name: &String) -> Result<Vec<u8>, ()> {
        self.watcher
            .watch(&name, RecursiveMode::NonRecursive)
            .unwrap();
        self.should_reload.insert(name.clone(), false);
        read(name).map_err(|_| ())
    }

    fn should_reload(&mut self, name: &String) -> bool {
        while let Ok(ev) = self.watcher_recv.try_recv() {
            //  We only care about file modifications.
            //  Maybe in the future, renames will also matter.
            match ev {
                DebouncedEvent::Write(name) => {
                    *self
                        .should_reload
                        .get_mut(&name.to_str().unwrap().to_string())
                        .unwrap() = true
                }
                _ => {}
            }
        }

        *self.should_reload.get(name).unwrap_or(&false)
    }
}
