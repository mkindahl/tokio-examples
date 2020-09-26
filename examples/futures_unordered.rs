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

// An example creating a bunch of futures and then collect them using
// `FuturesUnordered`. Add printouts to see that they are spawed in
// the right order, but complete in a different order.

use futures::executor::block_on_stream;
use futures::stream::FuturesUnordered;
use std::time::Duration;
use tokio;

struct Item {
    number: u64,
    resolved: bool,
}

impl Item {
    async fn resolve(&mut self) {
        // Delay the tasks before allowing them to resolve. Compute
        // the day in a weird way so that it will resolve the tasks in
        // a different order compared to how they were added.
        tokio::time::delay_for(Duration::from_millis(5 * (10 - self.number) % 21)).await;
        self.resolved = true;
    }

    fn print_result(&self) {
        if self.resolved {
            println!("task {} resolved", self.number);
        } else {
            println!("task {} not resolved", self.number);
        }
    }
}

#[tokio::main]
async fn main() {
    let items: Vec<_> = (0..10)
        .map(|n| Item {
            number: n,
            resolved: false,
        })
        .collect();

    // This is a stream. We cannot directly iterate over it.
    let tasks: FuturesUnordered<_> = items
        .into_iter()
        .map(|mut item| async move {
            println!("task {} spawned", item.number);
            item.resolve().await;
            item
        })
        .collect();

    // Turn the stream into a blocking iterator on the stream and wait
    // for them to complete.
    for item in block_on_stream(tasks) {
        item.print_result();
    }
}
