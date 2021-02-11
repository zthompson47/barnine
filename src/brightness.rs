use std::path::Path;

use tokio::fs::read_to_string;
use zbus::azync::Connection as Dbus;

use crate::err::Error;

#[derive(Debug)]
pub enum Brightness {
    Keyboard(Delta),
    Screen(Delta),
}

#[derive(Debug)]
pub enum Delta {
    UpPct(u32),
    DownPct(u32),
}

pub async fn brighten(update: Brightness) -> Result<u32, Error> {
    match update {
        Brightness::Keyboard(delta) => {
            let (brt, brt_max) = cur_brt_with_max("leds", "smc::kbd_backlight").await;
            let new_brt = match delta {
                Delta::UpPct(amt) => brt + brt_max * amt / 100,
                Delta::DownPct(amt) => brt - brt_max * amt / 100,
            };

            let connection = Dbus::new_system().await?;
            connection.call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/session/auto",
                Some("org.freedesktop.login1.Session"),
                "SetBrightness",
                &("leds", "smc::kbd_backlight", new_brt),
            ).await?;
            Ok(new_brt)
        },
        Brightness::Screen(delta) => {
            let (brt, brt_max) = cur_brt_with_max("backlight", "intel_backlight").await;
            let new_brt = match delta {
                Delta::UpPct(amt) => brt + brt_max * amt / 100,
                Delta::DownPct(amt) => brt - brt_max * amt / 100,
            };

            let connection = Dbus::new_system().await?;
            connection.call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1/session/auto",
                Some("org.freedesktop.login1.Session"),
                "SetBrightness",
                &("backlight", "intel_backlight", new_brt),
            ).await?;
            Ok(new_brt)
        },
    }
}

async fn cur_brt_with_max(device_type: &str, device: &str) -> (u32, u32) {
    let brt_file = Path::new("/sys/class")
        .join(device_type)
        .join(device)
        .join("brightness");
    let brt = read_to_string(brt_file).await.unwrap();

    let brt_max_file = Path::new("/sys/class")
        .join(device_type)
        .join(device)
        .join("max_brightness");
    let brt_max = read_to_string(brt_max_file).await.unwrap();

    let brt = brt.trim();
    let brt_max = brt_max.trim();

    let brt = brt.parse::<u32>().unwrap();
    let brt_max = brt_max.parse::<u32>().unwrap();

    (brt, brt_max)
}
