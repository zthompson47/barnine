use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

use barnine::rpc::socket_path;

fn main() -> std::io::Result<()> {
    if let Some(cmd) = env::args().nth(1) {
        let mut stream = UnixStream::connect(socket_path())?;
        stream.write_all(cmd.as_bytes())?;
    }
    Ok(())
}
