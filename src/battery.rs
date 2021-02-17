use std::time::Duration;

use tokio::fs::read_to_string;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;

use crate::bar::Update;
use crate::err::Res;

const BATTERY_UEVENT: &str = "/sys/class/power_supply/BAT0/uevent";

#[derive(Default)]
pub struct Battery {
    pub status: Option<String>,
    pub capacity: Option<String>,
}

pub async fn get_battery(tx: UnboundedSender<Update>) -> Res<()> {
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

        tx.send(Update::BatteryStatus(battery.status))?;
        tx.send(Update::BatteryCapacity(battery.capacity))?;
        tx.send(Update::Redraw)?;

        sleep(Duration::from_secs(5)).await;
    }
}
