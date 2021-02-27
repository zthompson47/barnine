use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

use notify::event::DataChange::Any;
use notify::event::ModifyKind::Data;
use notify::EventKind::Modify;
use notify::RecursiveMode::NonRecursive;
use notify::{Event, RecommendedWatcher, Watcher};
use serde_derive::Deserialize;
use tokio::fs;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::time::{sleep, Duration};
use tracing::debug;

use crate::bar::{Block, Update};
use crate::err::Res;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default: RefCell<Block>,
    pub bar: Vec<RefCell<Block>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: RefCell::new(Block::default()),
            bar: Vec::new(),
        }
    }
}

pub async fn watch_config(tx_updates: UnboundedSender<Update>) -> Res<()> {
    debug!("in watch_config");
    let (tx, mut rx) = unbounded_channel::<()>();

    thread::spawn(move || {
        debug!("in watch_config thread");
        let (tx_local, rx_local) = channel::<()>();
        let mut config: RecommendedWatcher;
        config = Watcher::new_immediate(move |e| match e {
            Ok(event) => match event {
                Event {
                    kind: Modify(Data(Any)),
                    ..
                } => tx_local.send(()).unwrap(),
                _ => {
                    debug!("got event-->>{:?}", event);
                }
            },
            _ => {}
        })
        .unwrap();
        debug!(
            "about to watch config file:{:?}",
            get_config_file("barnine").unwrap()
        );
        config
            .watch(
                get_config_file("barnine").unwrap().parent().unwrap(),
                NonRecursive,
            )
            .unwrap();
        while let Ok(()) = rx_local.recv() {
            debug!("got Modify(Data( signal");
            tx.send(()).unwrap();
        }
    });

    loop {
        while let Some(()) = rx.recv().await {
            let config_file = get_config_file("barnine").unwrap();
            if config_file.is_file() {
                let toml: String = fs::read_to_string(&config_file).await.unwrap();
                debug!("abou to -------------------!!!!!11--------------------");
                let config: Result<Config, _> = toml::from_str(&toml);
                debug!("{:#?}", config);
                if config.is_err() {
                    break;
                }
                tx_updates.send(Update::Config(config.unwrap())).unwrap();
                tx_updates.send(Update::Redraw).unwrap();
            }
        }
        sleep(Duration::from_millis(200)).await;
    }
}

fn get_config_file(app_name: &str) -> Res<Box<Path>> {
    let mut config_path = match env::var("XDG_CONFIG_HOME") {
        Ok(dir) => Path::new(&dir).join(app_name),
        Err(_) => match env::var("HOME") {
            Ok(dir) => Path::new(&dir).join(".config").join(app_name),
            Err(_) => Path::new("/tmp").join(app_name),
        },
    };
    config_path.set_extension("toml");
    Ok(config_path.into_boxed_path())
}
