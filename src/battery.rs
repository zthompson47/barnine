use std::path::Path;
use std::time::Duration;

use tokio::fs::read_to_string;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::interval;

use crate::bar::Update;
use crate::err::Res;

const BAT0: &str = "/sys/class/power_supply/BAT0";

#[derive(Debug)]
pub struct Battery {
    pub status: String,
    pub capacity: String,
}

pub async fn get_battery(tx: UnboundedSender<Update>) -> Res<()> {
    let mut idle = interval(Duration::from_secs(5));

    loop {
        idle.tick().await;

        let battery = Battery {
            status: {
                let path = Path::new(BAT0).join("status");
                read_to_string(path)
                    .await
                    .unwrap()
                    .trim()
                    .to_string()
            },
            capacity: {
                let path = Path::new(BAT0).join("capacity");
                read_to_string(path)
                    .await
                    .unwrap()
                    .trim()
                    .to_string()
            },
        };

        tx.send(Update::BatteryStatus(Some(battery.status)))?;
        tx.send(Update::BatteryCapacity(Some(battery.capacity)))?;
        tx.send(Update::Redraw)?;
    }
}
