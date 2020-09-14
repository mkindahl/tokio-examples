// Copyright 2019 Mats Kindahl
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

// Since it is not possible to clone sockets, it is necessary to have
// dedicated futures for writing the sockets. Reading the sockets
// should still go to a single future, so this is to handle multiple
// futures that want to send data.
//
// In the example, we will spawn one task that read data from the
// socket and send back a processed response, one task that has to
// role of sending out packets accepted on a channel, and a periodic
// thread that send another message on the UDP socket using the
// channel.

use futures::prelude::*;
use std::env;
use std::net::SocketAddr;
use std::result::Result;
use std::str::from_utf8;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::{io, join};

struct Message {
    buf: String,
    dest: Option<SocketAddr>,
}

#[derive(Debug)]
enum Error {
    GenericError(String),
    EncodingError(std::str::Utf8Error),
    IOError(std::io::Error),
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Error {
        Error::EncodingError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IOError(error)
    }
}

impl<M, A> From<mpsc::error::TrySendError<(M, A)>> for Error {
    fn from(error: mpsc::error::TrySendError<(M, A)>) -> Error {
        Error::GenericError(format!("{}", error))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let socket = {
        let addr = address.parse::<SocketAddr>()?;
        UdpSocket::bind(&addr).await?
    };

    // Here we split the socket into the sender and receiver side. We
    // cannot clone the sender (writer) side, so we have to handle
    // this using an mpsc channel.
    let (mut reader, mut writer) = socket.split();

    // Create an mpsc channel to use for internal communication. We
    // can clone the writer side of the channel (tx), so we are going
    // to use that later.
    let (mut tx, mut rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(10);

    // This is the transmitting task that will transmit anything that
    // arrives on the channel. The socket address is optional, and if
    // none is provided, the last used address will be used.
    let transmitter_task = {
        let mut last_address: Option<SocketAddr> = None;
        async move {
            while let Some(msg) = rx.recv().await {
                let address = match msg.dest {
                    Some(addr) => Some(addr),
                    None => last_address,
                };
                last_address = address;
                if let Some(dest) = address {
                    let packet = format!("FYI - {}\n", msg.buf);
                    writer.send_to(packet.as_bytes(), &dest).await?;
                }
            }
            Ok::<_, io::Error>(())
        }
    };

    // This is the receiver task that handles all incoming
    // packets. They are just relayed to the transmitter task.
    let receiver_task = {
        let mut tx = tx.clone();
        async move {
            let mut buf = vec![0; 128];
            loop {
                let (count, addr) = reader.recv_from(&mut buf).await?;
                if count == 0 {
                    break;
                }
                let msg = Message {
                    buf: format!("Simon says: '{}'", from_utf8(&buf).unwrap()),
                    dest: Some(addr),
                };
                if let Err(err) = tx.send(msg).await {
                    println!("Error: {}", err);
                    break;
                }
            }
            Ok::<_, io::Error>(())
        }
    };

    // This is a regular task that just inject a message into the
    // queue of the transmitter task. We use it to demonstrate how to
    // send messages on the same socket from multiple closures.
    let injector_task = {
        async move {
            let mut seconds: i32 = 1;
            let mut ticks = interval(Duration::from_millis(1000));
            while let Some(_interval) = ticks.next().await {
                seconds += 1;
                let msg = Message {
                    buf: format!("{} seconds passed", seconds),
                    dest: None,
                };
                if let Err(err) = tx.send(msg).await {
                    println!("Error: {}", err);
                    break;
                }
            }
            Ok::<_, io::Error>(())
        }
    };

    let _ = join!(
        tokio::spawn(transmitter_task),
        tokio::spawn(receiver_task),
        tokio::spawn(injector_task),
    );

    Ok(())
}
