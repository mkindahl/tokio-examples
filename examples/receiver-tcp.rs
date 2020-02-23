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

//! Example application that receives messages over TCP.
//!
//! This application will receive messages over TCP and print them to
//! the terminal. It will listen on port 6148 and spawn a session for
//! any incoming TCP connection, read the messages until the
//! connection is shut down.

use std::error::Error;
use std::str::from_utf8;
use tokio;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[tokio::main(core_threads = 5)]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:6142").await?;
    println!("Listening on: {}", listener.local_addr()?);
    while let Ok((mut socket, addr)) = listener.accept().await {
        println!("Accepting: {}", addr);
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                match from_utf8(&buf) {
                    Ok(msg) => println!("received: {}", msg),
                    Err(err) => println!("error: {}", err),
                }
            }
        });
    }
    Ok(())
}
