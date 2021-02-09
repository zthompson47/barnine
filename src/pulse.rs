use std::time::Duration;

use async_std::task::sleep;
use log::debug;
use smol::channel::Sender;
use zbus::{azync::Connection, Result};
use zbus_macros::dbus_proxy;
use zvariant::ObjectPath;

use crate::bar::Update;
use crate::err::Res;

pub async fn get_pulse(tx: Sender<Update>) -> Res<()> {
    loop {
        tx.send(Update::Volume(volume().await.ok())).await?;
        tx.send(Update::Redraw).await?;
        sleep(Duration::from_secs(5)).await;
    }
}

pub async fn set_volume(vol: u32) -> Res<()> {
    debug!("<>--> enter set_volume, vol:{}", vol);
    let pulse_conn = new_pulse_connection().await?;
    debug!("<>--> got pulse conn");

    let core_proxy = AsyncPulseCoreProxy::new(&pulse_conn)?;
    debug!("<>--> got pulse core_proxy");
    let sinks = core_proxy.sinks().await?;
    debug!("<>--> got pulse sinks");

    for sink in sinks.iter().map(|s| s.to_string()) {
        debug!("<>--> in sink");
        let sink_proxy = AsyncSinkProxy::new_for_path(&pulse_conn, sink).unwrap();
        let mut new_volume = Vec::new();
        new_volume.push(vol);
        debug!("<>--> SET_new volume:{:?}", new_volume);
        match sink_proxy.set_volume(new_volume).await {
            Ok(val) => debug!("Ok: {:?}", val),
            Err(err) => debug!("Err: {:?}", err),
        }

        debug!("<>--> after set_volume______");
    }
    Ok(())
}

pub async fn volume() -> Res<u32> {
    debug!("()()))(()))>> enter volume()");
    let pulse_conn = new_pulse_connection().await?;
    debug!("()()))(()))>> got pulse conn");

    let core_proxy = AsyncPulseCoreProxy::new(&pulse_conn)?;
    debug!("()()))(()))>> got core_proxy");
    let sinks = core_proxy.sinks().await?;
    debug!("()()))(()))>> got sinks");

    for sink in sinks.iter().map(|s| s.to_string()) {
        let sink_proxy = AsyncSinkProxy::new_for_path(&pulse_conn, sink).unwrap();
        debug!("()()))(()))>> got sink proxy");
        let vol = sink_proxy.volume().await.unwrap();
        debug!("()()))(()))>> got volume {:?}", vol);
        /*
        let vol: Vec<u32> = vol
            .iter()
            .map(|x| x * 100 / 65536)
            .collect();
        */
        return Ok(vol[0]);
    }
    Err("No sink found".into())
}

async fn new_pulse_connection() -> Res<Connection> {
    let conn = Connection::new_session().await?;
    let addr_lookup = AsyncPulseAddressProxy::new(&conn)?;
    let addr = addr_lookup.address().await?;
    Ok(Connection::new_for_address(&addr, false).await?)
}

pub async fn pulse_info() -> Vec<String> {
    let mut result = Vec::new();

    let conn = Connection::new_session().await.unwrap();
    let p = AsyncPulseAddressProxy::new(&conn).unwrap();
    let addr = p.address().await.unwrap();
    result.push(format!("{}", addr));

    let pulse_conn = Connection::new_for_address(&addr, false).await.unwrap();
    let c = AsyncPulseCoreProxy::new(&pulse_conn).unwrap();
    let name = c.name().await.unwrap();
    let version = c.version().await.unwrap();
    let dsf = c.default_sample_format().await.unwrap();
    let dsr = c.default_sample_rate().await.unwrap();
    let sinks = c.sinks().await.unwrap();

    result.push(format!("{} {}", name, version));
    result.push(format!("dsr: {}", dsr));
    result.push(format!("dsf: {:?}", SampleFormat::from(dsf)));
    result.push(String::from("Sinks:"));

    for sink in sinks.iter().map(|s| s.to_string()) {
        let s = AsyncSinkProxy::new_for_path(&pulse_conn, sink).unwrap();
        result.push(format!("name: {}", s.name().await.unwrap()));

        let vol = s.volume().await.unwrap();
        let vol: Vec<u32> = vol
            .iter()
            .map(|x| x * 100 / 65536)
            .collect();
        result.push(format!("volume: {:?}", vol));

        let sf = s.sample_format().await.unwrap();
        result.push(format!("format: {:?}", SampleFormat::from(sf)));
        result.push(format!("rate: {:?}", s.sample_rate().await.unwrap()));
        result.push(format!("channels: {:?}", s.channels().await.unwrap()));

        result.push(format!(
            "flat vol?: {:?}",
            s.has_flat_volume().await.unwrap()
        ));
        result.push(format!("base vol: {:?}", s.base_volume().await.unwrap()));
        result.push(format!("vol steps: {:?}", s.volume_steps().await.unwrap()));
        result.push(format!("mute: {:?}", s.mute().await.unwrap()));
        result.push(format!(
            "cfg latency: {:?}",
            s.configured_latency().await.unwrap()
        ));

        result.push(format!("latency: {:?}", s.latency().await.unwrap()));
        let ds = s.state().await.unwrap();
        result.push(format!("state: {:?}", DeviceState::from(ds)));
        result.push(format!(
            "hardware vol: {:?}",
            s.has_hardware_volume().await.unwrap()
        ));
        result.push(format!(
            "hardware mute: {:?}",
            s.has_hardware_mute().await.unwrap()
        ));
    }

    result
}

#[derive(Debug)]
enum DeviceState {
    /*
    Running, the device is being used by at least one non-corked stream.
    Idle, the device is active, but no non-corked streams are connected to it.
    Suspended, the device is not in use and may be currently closed.
    */
    Running,
    Idle,
    Suspended,
    None,
}

impl From<u32> for DeviceState {
    fn from(state: u32) -> Self {
        match state {
            0 => DeviceState::Running,
            1 => DeviceState::Idle,
            2 => DeviceState::Suspended,
            _ => DeviceState::None,
        }
    }
}

#[derive(Debug)]
enum SampleFormat {
    /*
    0 : Unsigned 8 bit PCM
    1 : 8 bit a-Law
    2 : 8 bit mu-Law
    3 : Signed 16 bit PCM, little endian
    4 : Signed 16 bit PCM, big endian
    5 : 32 bit IEEE floating point, little endian, range -1.0 to 1.0
    6 : 32 bit IEEE floating point, big endian, range -1.0 to 1.0
    7 : Signed 32 bit PCM, little endian
    8 : Signed 32 bit PCM, big endian
    9 : Signed 24 bit PCM packed, little endian
    10 : Signed 24 bit PCM packed, big endian
    11 : Signed 24 bit PCM in LSB of 32 bit words, little endian
    12 : Signed 24 bit PCM in LSB of 32 bit words, big endian
    */
    S16le,
    None,
}

impl From<u32> for SampleFormat {
    fn from(index: u32) -> Self {
        match index {
            3 => SampleFormat::S16le,
            _ => SampleFormat::None,
        }
    }
}

#[dbus_proxy(
    interface = "org.PulseAudio.ServerLookup1",
    default_service = "org.PulseAudio1",
    default_path = "/org/pulseaudio/server_lookup1"
)]
trait PulseAddress {
    #[dbus_proxy(property)]
    fn address(&self) -> Result<String>;
}

#[dbus_proxy(
    interface = "org.PulseAudio.Core1",
    default_service = "org.PulseAudio.Core1",
    default_path = "/org/pulseaudio/core1"
)]
trait PulseCore {
    #[dbus_proxy(property)]
    fn name(&self) -> Result<String>;
    #[dbus_proxy(property)]
    fn version(&self) -> Result<String>;
    #[dbus_proxy(property)]
    fn sinks(&self) -> Result<Vec<ObjectPath<'_>>>;
    #[dbus_proxy(property)]
    fn default_sample_format(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn default_sample_rate(&self) -> Result<u32>;
}

#[dbus_proxy(
    interface = "org.PulseAudio.Core1.Device",
    default_service = "org.PulseAudio.Core1.Device",
)]
trait Sink {
    #[dbus_proxy(property)]
    fn name(&self) -> Result<String>;
    #[dbus_proxy(property)]
    fn volume(&self) -> Result<Vec<u32>>;
    #[dbus_proxy(property)]
    fn sample_format(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn sample_rate(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn channels(&self) -> Result<Vec<u32>>;
    #[dbus_proxy(property)]
    fn has_flat_volume(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn base_volume(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn volume_steps(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn mute(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn configured_latency(&self) -> Result<u64>;
    #[dbus_proxy(property)]
    fn latency(&self) -> Result<u64>;
    #[dbus_proxy(property)]
    fn state(&self) -> Result<u32>;
    #[dbus_proxy(property)]
    fn has_hardware_volume(&self) -> Result<bool>;
    #[dbus_proxy(property)]
    fn has_hardware_mute(&self) -> Result<bool>;

    #[dbus_proxy(property)]
    fn set_volume(&self, vols: Vec<u32>) -> Result<()>;
}
