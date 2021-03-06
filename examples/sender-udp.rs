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

use std::env;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let message = env::args()
        .nth(1)
        .unwrap_or_else(|| "hello world".to_string());
    let addr: SocketAddr = "127.0.0.1:6142".parse()?;
    let mut socket = UdpSocket::bind("0.0.0.0:0").await?;
    let result = socket.send_to(message.as_bytes(), &addr).await?;
    println!("wrote to stream: result={:?}", result);
    Ok(())
}
