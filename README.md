# Rust Tokio Examples

This repository contain various test programs and examples of using
Tokio and are mainly intended to be educational.

Cloning and unpacking the repository:
```shell
git clone https://github.com/mkindahl/tokio-examples
cd tokio-examples
```

## Running examples

To run the `cycle_stream` example (for example):
```shell
cargo run --example cycle_stream
```

### Socket manager

Socket manager implement a simple socket manager that receives
messages over a channel that it then sends. This example is introduced
since it is not possible to clone the writing side of a socket (even
though it is supported by the operating system) so it is necessary to
handle this using a channel instead.

It consists of a reader that reads message arriving on a socket, adds
some framing to it, and sends it back. In addition, there is an
injector task that regularly send a time message to the transmitter
task.

You can run this using
```shell
cargo run --example socket_manager -- 127.0.0.1:8080
```

Try something like this and see what happends:
```shell
nc -u localhost 8080
```

### Sending and receiving UDP

The two examples `sender-udp` and `receiver-udp` experiment with how
to send and received UDP packages. They are intentionally very simple
and only serve as the basis for other solutions.

`receiver-udp` will receive messages over UDP and print them to the
terminal. It will listen on port 6148 and spawn a session for any
incoming UDP connection, read the messages until the connection is
shut down.

You can start it using:
```bash
cargo run --example receiver-udp
```

The `sender-udp` program sends UDP packages. You can either start the
`receiver-udp` or listen for the message using `netcat`:

```bash
$ nc -l 6142
```

To send a message, the `sender-udp` accept a message on the command
line that it will send to the port 6142 using a UDP connection.

```bash
$ cargo run --example sender-udp 'just a test'
```

If no message is provided, "hello world" will be used.

