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

//! Example application that receives messages over UDP.
//!
//! This application will receive messages over UDP and print them to
//! the terminal. It will listen on port 6148 and spawn a session for
//! any incoming UDP connection, read the messages until the
//! connection is shut down.
//!
//! You can start it using:
//! ```bash
//! cargo run --example receiver-udp
//! ```

use std::error::Error;
use std::str::from_utf8;
use tokio;
use tokio::net::UdpSocket;

#[tokio::main(core_threads = 5)]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut socket = UdpSocket::bind("0.0.0.0:6142").await?;
    let mut buf = [0; 1024];
    while let Ok((bytes, addr)) = socket.recv_from(&mut buf).await {
        print!("Packet of {} bytes from {}: ", bytes, addr);
        match from_utf8(&buf[0..bytes]) {
            Ok(msg) => println!("{}", msg),
            Err(err) => println!("ERROR {}", err),
        }
    }
    Ok(())
}
