use std::time::Duration;

use async_std::task::sleep;
use smol::channel;
use smol::fs::read_to_string;

use crate::err::Res;
use crate::bar::Update;

const BATTERY_UEVENT: &str = "/sys/class/power_supply/BAT0/uevent";

#[derive(Default)]
pub struct Battery {
    pub status: Option<String>,
    pub capacity: Option<String>,
}

pub async fn get_battery(tx: channel::Sender<Update>) -> Res<()> {
    loop {
        let file = read_to_string(BATTERY_UEVENT).await?;
        let mut battery = Battery::default();

        for line in file.split('\n') {
            let v: Vec<&str> = line.split('=').collect();
            match v[0] {
                "POWER_SUPPLY_STATUS" => battery.status = Some(v[1].to_string()),
                "POWER_SUPPLY_CAPACITY" => battery.capacity = Some(v[1].to_string()),
                _ => {}
            }
        }

        tx.send(Update::BatteryStatus(battery.status)).await?;
        tx.send(Update::BatteryCapacity(battery.capacity)).await?;
        tx.send(Update::Redraw).await?;

        sleep(Duration::from_secs(5)).await;
    }
}
