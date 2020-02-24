// Copyright 2020 Mats Kindahl
//
// Licensed under the Apache License, Version 2.0 (the "License"); you
// may not use this file except in compliance with the License.  You
// may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.  See the License for the specific language governing
// permissions and limitations under the License.

//! Application that acts split an incoming message into several other
//! messages.
//!
//! It will accept a connection attempt on port 6142 and open
//! connections to ports 6150-6152. Any messages sent to 6142 will
//! then be forwarded to ports 6150-6152 asynchronously. You can test
//! it by setting up three listening servers using `nc`, start the
//! intermediate, and then send a message on port 6142.
//!
//! ```bash
//! bash-1$ nc -l 6150
//! bash-2$ nc -l 6151
//! bash-3$ nc -l 6151
//! bash-4$ cargo run --example intermediate-tcp
//! bash-5$ cargo run --example sender-tcp 'just a test'
//! ```

use futures::prelude::*;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

#[tokio::main(core_threads = 5)]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:6142").await?;

    println!("Listening on: {}", listener.local_addr()?);
    while let Ok((mut socket, addr)) = listener.accept().await {
        println!("Accepting: {}", addr);
        let mut destinations = Vec::new();
        destinations.push(TcpStream::connect("127.0.0.1:6150").await?);
        destinations.push(TcpStream::connect("127.0.0.1:6151").await?);
        destinations.push(TcpStream::connect("127.0.0.1:6152").await?);

        let mut buf = [0; 1024];
        loop {
            let bytes = socket
                .read(&mut buf)
                .await
                .expect("failed to read data from socket");

            if bytes == 0 {
                break;
            }

            future::join_all(
                destinations
                    .iter_mut()
                    .map(|dest| dest.write_all(&buf[0..bytes])),
            )
            .await;
        }
    }
    Ok(())
}
