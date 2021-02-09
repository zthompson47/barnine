use log::debug;

use crate::{
    brightness::Delta,
    err::Res,
    pulse,
};

#[derive(Debug)]
pub enum Volume {
    Speakers(Delta),
}

pub async fn volume(update: Volume) -> Res<u32> {
    debug!("<><>  volume called:{:?}", update);
    match update {
        Volume::Speakers(delta) => {
            debug!("<><>  match Speakers");
            let delta:i32 = match delta {
                Delta::UpPct(val) => val as i32 * 65536 / 100,
                Delta::DownPct(val) => -(val as i32) * 65536 / 100,
            };
            debug!("<><> Speakers delta {}", delta);
            let cur_volume = pulse::volume().await?;
            let new_volume = cur_volume as i32 + delta;
            let new_volume: u32 = if new_volume < 0 {
                0
            } else {
                new_volume as u32
            };
            debug!("<><> got pulse delta:{} cur_volume:{} new_volume:{}", delta, cur_volume, new_volume);
            pulse::set_volume(new_volume).await?;
            Ok(new_volume)
        },
    }
}
