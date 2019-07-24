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

use bytes::Bytes;
use futures::prelude::*;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::env;
use std::net::SocketAddr;
use std::result::Result;
use std::str::from_utf8;
use std::time::{Duration, Instant};
use tokio::codec::BytesCodec;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::timer::Interval;

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

impl<M, A> From<futures::sync::mpsc::TrySendError<(M, A)>> for Error {
    fn from(error: futures::sync::mpsc::TrySendError<(M, A)>) -> Error {
        Error::GenericError(format!("{}", error))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let socket = {
        let addr = address.parse::<SocketAddr>()?;
        UdpSocket::bind(&addr)?
    };

    // Here we split the socket into the sender and receiver side. We
    // cannot clone the sender (writer) side, so we have to handle
    // this using an mpsc channel.
    let (mut writer, reader) = UdpFramed::new(socket, BytesCodec::new()).split();

    // Create an mpsc channel to use for internal communication. We
    // can clone the writer side of the channel (tx), so we are going
    // to use that later.
    let (tx, rx) = mpsc::channel(1024);

    // This is the transmitting task that will transmit anything that
    // arrives on the channel. The socket address is optional, and if
    // none is provided, the last used address will be used.
    let transmitter_task = {
        let mut last_address: Option<SocketAddr> = None;
        rx.for_each(move |(msg, dest): (String, Option<SocketAddr>)| {
            let address = match dest {
                Some(addr) => Some(addr),
                None => last_address,
            };
            last_address = address;
            if let Some(addr) = address {
                // start_send() + poll_complete() is used here since
                // send() is FnOnce, while start_send() and
                // poll_complete() are FnMut.
                writer
                    .start_send((Bytes::from(msg), addr))
                    .map_err(|_| ())?;
                writer.poll_complete().map_err(|_| ())?;
            }
            Ok(())
        })
        .map_err(|err| println!("error: {:?}", err))
    };

    // This is the receiver task that handles all incoming
    // packets.
    let receiver_task = {
        let mut tx = tx.clone();
        reader
            .map_err(|e| Error::IOError(e))
            .for_each(move |(msg, addr)| {
                let msg = format!("Simon says: {}", from_utf8(&msg)?);
                tx.try_send((msg, Some(addr)))?;
                Ok(())
            })
            .map_err(|err| println!("error: {:?}", err))
            .map(|_| ())
    };

    // This is a regular task that just inject a message into the
    // queue. We use it to demonstrate how to send messages on the
    // same socket from multiple closures.
    let injector_task = {
        let mut tx = tx.clone();
        let mut seconds = 1;
        Interval::new(Instant::now(), Duration::from_millis(1000))
            // This is needed to use our version of Error rather than
            // IntervalError.
            .map_err(|err| Error::GenericError(format!("interval error: {}", err)))
            .for_each(move |_| {
                seconds += 1;
                tx.try_send((format!("{} seconds passed\n", seconds), None))?;
                Ok(())
            })
            .map_err(|e| panic!("first - interval errored; err={:?}", e))
    };

    tokio::run(futures::lazy(|| {
        tokio::spawn(transmitter_task);
        tokio::spawn(receiver_task);
        tokio::spawn(injector_task);
        Ok(())
    }));
    Ok(())
}
