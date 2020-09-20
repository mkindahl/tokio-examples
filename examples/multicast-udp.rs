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

//use futures::future;
use bytes::Bytes;
use futures::executor::block_on_stream;
use futures::stream::FuturesUnordered;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::UdpSocket;

struct Connection {
    socket: UdpSocket,
    addr: SocketAddr,
}

/// Multicast the packet to all the sockets and collect sockets after.
async fn multicast_packet(
    packet: Bytes,
    addresses: Vec<Connection>,
) -> io::Result<Vec<Connection>> {
    let tasks: FuturesUnordered<_> = addresses
        .into_iter()
        .map(|mut conn| {
            let packet = packet.clone();
            async move {
                conn.socket.send_to(&packet, conn.addr).await?;
                Ok(conn)
            }
        })
        .collect();
    block_on_stream(tasks).collect()
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut incoming = UdpSocket::bind("0.0.0.0:6142").await?;
    let mut outbound = vec![];
    for arg in env::args().skip(1) {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let addr = arg.parse()?;
        outbound.push(Connection { socket, addr });
    }

    loop {
        let mut buf = [0; 1500];
        println!("Waiting for packet");
        match incoming.recv(&mut buf).await {
            Ok(bytes) if bytes == 0 => break,
            Ok(bytes) => {
                let packet = Bytes::copy_from_slice(&buf[0..bytes]);
                outbound = multicast_packet(packet, outbound).await?;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
