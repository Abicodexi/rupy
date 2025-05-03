use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub struct AssetWatcher {
    _watcher: RecommendedWatcher, // Keep alive
}

impl AssetWatcher {
    pub fn new<F>(watch_path: PathBuf, mut on_change: F) -> anyhow::Result<Self>
    where
        F: FnMut(Event) + Send + 'static,
    {
        let (tx, rx) = channel();
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        watcher.watch(&watch_path, RecursiveMode::Recursive)?;

        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                on_change(event.unwrap());
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
