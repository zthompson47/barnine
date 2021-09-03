use std::cell::RefCell;
use std::env;
use std::path::Path;

use notify::event::DataChange::Any;
use notify::event::ModifyKind::Data;
use notify::EventKind::Modify;
use notify::RecursiveMode::NonRecursive;
use notify::{Event, RecommendedWatcher, Watcher};
use serde_derive::Deserialize;
use tokio::fs;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
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
    // TODO create config dir and sample barnine.toml if absent
    debug!("in watch_config thread");
    let (tx_watcher, mut rx_watcher) = unbounded_channel::<()>();

    let mut config: RecommendedWatcher = Watcher::new_immediate(move |e|


    //match e {
    if let Ok(event) = e {
        if let Event {
            kind: Modify(Data(Any)),
            paths: ref p,
            ..
        } = event
        {
            // Confirm that the modification is on the watched file
            let watched_file = get_config_file("barnine").unwrap();
            if p.contains(&std::path::PathBuf::from(watched_file)) {
                debug!("got event-->>{:?}", event);
                tx_watcher.send(()).unwrap();
            }
        }
    }
        //_ => {}




    )
    .unwrap();

    debug!(
        "about to watch config file:{:?}",
        get_config_file("barnine").unwrap()
    );

    config
        .watch(
            // TODO why do I need to monitor parent and not the file..
            //      with just the file: no Modify(Data(Any)) received.. ?!?
            get_config_file("barnine").unwrap().parent().unwrap(),
            NonRecursive,
        )
        .unwrap();

    // Load config file at startup
    send_config_update("barnine", tx_updates.clone()).await?;

    while let Some(()) = rx_watcher.recv().await {
        debug!("got Modify(Data()) recv");
        send_config_update("barnine", tx_updates.clone()).await?;
    }

    Ok(())
}

async fn send_config_update(app_name: &str, tx_updates: UnboundedSender<Update>) -> Res<()> {
    let config_file = get_config_file(app_name).unwrap();
    if config_file.is_file() {
        let toml: String = fs::read_to_string(&config_file).await.unwrap();
        let config: Result<Config, _> = toml::from_str(&toml);
        /* TODO
        if config.is_err() {
            break;
        }
        */
        tx_updates
            .send(Update::Config(Box::new(config.unwrap())))
            .unwrap();
        tx_updates.send(Update::Redraw).unwrap();
    }

    Ok(())
}

fn get_config_file(app_name: &str) -> Res<Box<Path>> {
    // Look for APPNAME_DEV_DIR environment variable to override default
    let mut dev_dir = app_name.to_uppercase();
    dev_dir.push_str("_DEV_DIR");

    let mut config_path = match env::var(&dev_dir) {
        Ok(dev_dir) => Path::new(&dev_dir).join(app_name),
        Err(_) => match env::var("XDG_CONFIG_HOME") {
            Ok(dir) => Path::new(&dir).join(app_name).join(app_name),
            Err(_) => match env::var("HOME") {
                Ok(dir) => Path::new(&dir)
                    .join(".config")
                    .join(app_name)
                    .join(app_name),
                Err(_) => Path::new("/tmp").join(app_name),
            },
        },
    };
    config_path.set_extension("toml");

    Ok(config_path.into_boxed_path())
}
