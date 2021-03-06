use std::env;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

use barnine::rpc::get_socket_path;

fn main() -> std::io::Result<()> {
    if let Some(cmd) = env::args().nth(1) {
        let sock = get_socket_path("barnine");
        let mut stream = UnixStream::connect(sock)?;
        stream.write_all(cmd.as_bytes())?;
    }
    Ok(())
}
