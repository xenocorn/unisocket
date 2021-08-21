# Unisocket
Fork of [this](https://crates.io/crates/multisock) library with with minor improvements.

This crate provides unified `SocketAddr`, `Stream` and `Listener` types that work with both TCP
and UNIX sockets.

Many applications don't really care whether they are connecting to a UNIX or TCP service, they
simply want to use the service. Similarly, applications may want to provide a service over
either a UNIX socket or TCP port. The difference between these two socket types matters as much
to application logic as the difference between IPv4 and IPv6 - not that much, typically. Yet
libstd provides a unified type for IPv4 and IPv6 sockets, but requires a separate type for UNIX
sockets. The types provided by this crate allow for writing socket-type-agnostic network
applications that treat UNIX sockets in the same way as IPv4 and IPv6: Just a matter of
run-time configuration.

These types should behave the same as the `SocketAddr`, `TcpStream`/`UnixStream` and
`TcpListener`/`UnixListener` in libstd. There is currently no support for mio or tokio.

UDP and Datagram sockets are not currently supported.

On Windows, these types only support TCP and are just lightweight wrappers around TCP sockets.