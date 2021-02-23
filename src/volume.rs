use tracing::trace;

use crate::{brightness::Delta, err::Res, pulse};

#[derive(Debug)]
pub enum Volume {
    Speakers(Delta),
}

pub async fn volume(update: Volume) -> Res<u32> {
    match update {
        Volume::Speakers(delta) => {
            let delta: i32 = match delta {
                Delta::UpPct(val) => val as i32 * 65536 / 100,
                Delta::DownPct(val) => -(val as i32) * 65536 / 100,
            };
            let cur_volume = pulse::get_volume().await?;
            let new_volume = cur_volume as i32 + delta;
            let new_volume: u32 = if new_volume < 0 { 0 } else { new_volume as u32 };
            trace!(
                "<><> got pulse delta:{} cur_volume:{} new_volume:{}",
                delta, cur_volume, new_volume
            );
            pulse::set_volume(new_volume).await?;
            Ok(new_volume)
        }
    }
}
