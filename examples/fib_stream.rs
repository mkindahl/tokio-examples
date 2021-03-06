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

// Example to demonstrate how to generate an infinite stream of
// fibonacci numbers. Not that this stream will end with an overflow.

extern crate tokio_examples;

use futures::StreamExt;
use tokio_examples::fibonacci;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let numbers = fibonacci();
    tokio::pin!(numbers);
    while let Some(number) = numbers.next().await {
        println!("number: {}", number);
    }
    Ok(())
}
