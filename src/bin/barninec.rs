use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

fn main() -> std::io::Result<()> {
    if let Some(cmd) = env::args().nth(1) {
        let mut stream = UnixStream::connect("/home/zach/barnine.sock")?;
        stream.write_all(cmd.as_bytes())?;
        // let mut response = String::new();
        // stream.read_to_string(&mut response)?;
        // println!("{}", response);
    }
    Ok(())
}
