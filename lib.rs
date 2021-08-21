//! This crate provides unified `SocketAddr`, `Stream` and `Listener` types that work with both TCP
//! and UNIX sockets.
//!
//! Many applications don't really care whether they are connecting to a UNIX or TCP service, they
//! simply want to use the service. Similarly, applications may want to provide a service over
//! either a UNIX socket or TCP port. The difference between these two socket types matters as much
//! to application logic as the difference between IPv4 and IPv6 - not that much, typically. Yet
//! libstd provides a unified type for IPv4 and IPv6 sockets, but requires a separate type for UNIX
//! sockets. The types provided by this crate allow for writing socket-type-agnostic network
//! applications that treat UNIX sockets in the same way as IPv4 and IPv6: Just a matter of
//! run-time configuration.
//!
//! These types should behave the same as the `SocketAddr`, `TcpStream`/`UnixStream` and
//! `TcpListener`/`UnixListener` in libstd. There is currently no support for mio or tokio.
//!
//! UDP and Datagram sockets are not currently supported.
//!
//! On Windows, these types only support TCP and are just lightweight wrappers around TCP sockets.

use std::io;
use std::net;
use std::fmt;
use std::time::Duration;
use std::str::FromStr;
#[cfg(unix)]
use std::path::{Path,PathBuf};
#[cfg(unix)]
use std::os::unix::net as unix;


/// Wrapper for a `std::net::SocketAddr` or UNIX socket path.
///
/// UNIX sockets are prefixed with 'unix:' when parsing and formatting.
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum SocketAddr {
    Inet(net::SocketAddr),
    #[cfg(unix)]
    Unix(PathBuf)
}

impl From<net::SocketAddr> for SocketAddr {
    fn from(s: net::SocketAddr) -> SocketAddr {
        SocketAddr::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::SocketAddr> for SocketAddr {
    fn from(s: unix::SocketAddr) -> SocketAddr {
        SocketAddr::Unix(match s.as_pathname() {
            None => Path::new("unnamed").to_path_buf(),
            Some(p) => p.to_path_buf()
        })
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SocketAddr::Inet(n) => write!(f, "{}", n),
            #[cfg(unix)]
            SocketAddr::Unix(n) => write!(f, "unix:{}", n.to_string_lossy())
        }
    }
}

impl FromStr for SocketAddr {
    type Err = net::AddrParseError;

    #[cfg(unix)]
    fn from_str(s: &str) -> Result<SocketAddr, net::AddrParseError> {
        if s.starts_with("unix:") {
            Ok(SocketAddr::Unix(Path::new(s.trim_start_matches("unix:")).to_path_buf()))
        } else {
            s.parse().map(SocketAddr::Inet)
        }
    }

    #[cfg(not(unix))]
    fn from_str(s: &str) -> Result<SocketAddr, net::AddrParseError> {
        s.parse().map(SocketAddr::Inet)
    }
}


impl SocketAddr {
    pub fn is_unix(&self) -> bool {
        match self {
            #[cfg(unix)]
            SocketAddr::Unix(_) => true,
            _ => false,
        }
    }
}




#[derive(Debug)]
pub enum Stream {
    Inet(net::TcpStream),
    #[cfg(unix)]
    Unix(unix::UnixStream)
}

impl From<net::TcpStream> for Stream {
    fn from(s: net::TcpStream) -> Stream {
        Stream::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::UnixStream> for Stream {
    fn from(s: unix::UnixStream) -> Stream {
        Stream::Unix(s)
    }
}

impl Stream {
    pub fn connect(s: &SocketAddr) -> io::Result<Stream> {
        match s {
            SocketAddr::Inet(s) => net::TcpStream::connect(s).map(Stream::Inet),
            #[cfg(unix)]
            SocketAddr::Unix(s) => unix::UnixStream::connect(s).map(Stream::Unix)
        }
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        match self {
            Stream::Inet(s) => s.local_addr().map(SocketAddr::Inet),
            #[cfg(unix)]
            Stream::Unix(s) => s.local_addr().map(|e| e.into())
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self {
            Stream::Inet(s) => s.peer_addr().map(SocketAddr::Inet),
            #[cfg(unix)]
            Stream::Unix(s) => s.peer_addr().map(|e| e.into())
        }
    }

    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        match self {
            Stream::Inet(s) => s.set_read_timeout(t),
            #[cfg(unix)]
            Stream::Unix(s) => s.set_read_timeout(t)
        }
    }

    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        match self {
            Stream::Inet(s) => s.set_write_timeout(t),
            #[cfg(unix)]
            Stream::Unix(s) => s.set_write_timeout(t)
        }
    }

    pub fn shutdown(&self, t: net::Shutdown) -> io::Result<()> {
        match self {
            Stream::Inet(s) => s.shutdown(t),
            #[cfg(unix)]
            Stream::Unix(s) => s.shutdown(t)
        }
    }

    pub fn try_clone(&self) -> io::Result<Self>{
        match self{
            Stream::Inet(stream) => {
                match stream.try_clone(){
                    Ok(new_stream) => {
                        Ok(Self::from(new_stream))
                    }
                    Err(err) => {Err(err)}
                }
            }
            #[cfg(unix)]
            Stream::Unix(stream) => {
                match stream.try_clone(){
                    Ok(new_stream) => {
                        Ok(Self::from(new_stream))
                    }
                    Err(err) => {Err(err)}
                }
            }
        }
    }
}

impl io::Read for &Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Stream::Inet(s) => (&mut (&*s)).read(buf),
            #[cfg(unix)]
            Stream::Unix(s) => (&mut (&*s)).read(buf)
        }
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut]) -> io::Result<usize> {
        match self {
            Stream::Inet(s) => (&mut (&*s)).read_vectored(bufs),
            #[cfg(unix)]
            Stream::Unix(s) => (&mut (&*s)).read_vectored(bufs)
        }
    }
}

impl io::Write for &Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Inet(s) => (&mut (&*s)).write(buf),
            #[cfg(unix)]
            Stream::Unix(s) => (&mut (&*s)).write(buf)
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> {
        match self {
            Stream::Inet(s) => (&mut (&*s)).write_vectored(bufs),
            #[cfg(unix)]
            Stream::Unix(s) => (&mut (&*s)).write_vectored(bufs)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Inet(s) => (&mut (&*s)).flush(),
            #[cfg(unix)]
            Stream::Unix(s) => (&mut (&*s)).flush()
        }
    }
}

impl io::Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { (&mut &*self).read(buf) }
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut]) -> io::Result<usize> { (&mut &*self).read_vectored(bufs) }
}

impl io::Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { (&mut &*self).write(buf) }
    fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> { (&mut &*self).write_vectored(bufs) }
    fn flush(&mut self) -> io::Result<()> { (&mut &*self).flush() }
}




#[derive(Debug)]
pub enum Listener {
    Inet(net::TcpListener),
    #[cfg(unix)]
    Unix(unix::UnixListener)
}

impl From<net::TcpListener> for Listener {
    fn from(s: net::TcpListener) -> Listener {
        Listener::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::UnixListener> for Listener {
    fn from(s: unix::UnixListener) -> Listener {
        Listener::Unix(s)
    }
}

impl Listener {
    pub fn bind(s: &SocketAddr) -> io::Result<Listener> {
        match s {
            SocketAddr::Inet(s) => net::TcpListener::bind(s).map(Listener::Inet),
            #[cfg(unix)]
            SocketAddr::Unix(s) => unix::UnixListener::bind(s).map(Listener::Unix)
        }
    }

    /// Same as `bind()`, but for UNIX sockets this will try to re-bind to the path if the process
    /// that used to listen to this address is no longer running. It can also optionally set the
    /// permissions of the UNIX socket.
    ///
    /// # Limitations
    ///
    /// Trying to bind to the same UNIX socket path from multiple processes is subject to a race
    /// condition.
    ///
    /// The permissions are set *after* performing the `bind()` operation, so if the default umask
    /// is less restrictive than the given mode, there is a short window where an unprivileged
    /// process could attempt to connect to the socket.
    pub fn bind_reuse(s: &SocketAddr, _mode: Option<u32>) -> io::Result<Listener> {
        let b = match (Self::bind(s), s) {
            #[cfg(unix)]
            (Err(ref e), &SocketAddr::Unix(ref p)) if e.kind() == io::ErrorKind::AddrInUse => {
                let e = io::Error::last_os_error();

                // Make sure it is a socket in the first place (we don't want to overwrite a
                // regular file)
                use std::os::unix::fs::FileTypeExt;
                match std::fs::symlink_metadata(p) {
                    Ok(ref m) if m.file_type().is_socket() => (),
                    _ => return Err(e),
                };

                // Try to connect to the socket to see if it's still alive.
                match Stream::connect(s) {
                    // Not alive, delete the socket and try to bind again.
                    Err(ref e2) if e2.kind() == io::ErrorKind::ConnectionRefused
                        => std::fs::remove_file(p).and_then(|_| Self::bind(s))?,
                    _ => return Err(e),
                }
            },
            (Err(e), _) => return Err(e),
            (Ok(l), _) => l,
        };

        #[cfg(unix)]
        #[allow(clippy::single_match)]
        match (_mode, s) {
            (Some(perm), &SocketAddr::Unix(ref p)) => {
                use std::fs::{set_permissions,Permissions};
                use std::os::unix::fs::PermissionsExt;
                set_permissions(p, Permissions::from_mode(perm))?;
            },
            _ => (),
        }
        Ok(b)
    }

    pub fn accept(&self) -> io::Result<(Stream,SocketAddr)> {
        match self {
            Listener::Inet(l) => l.accept().map(|(s,e)| (s.into(), e.into())),
            #[cfg(unix)]
            Listener::Unix(l) => l.accept().map(|(s,e)| (s.into(), e.into()))
        }
    }
}




#[test]
fn test_socket_addr_inet() {
    let ip4 = "127.0.0.1:10".parse::<net::SocketAddr>().unwrap();
    let ip6 = "[::20]:10".parse::<net::SocketAddr>().unwrap();
    assert_eq!(ip4.to_string(), "127.0.0.1:10".parse::<SocketAddr>().unwrap().to_string());
    assert_eq!(ip4.to_string(), SocketAddr::from(ip4).to_string());
    assert_eq!(ip6.to_string(), "[::20]:10".parse::<SocketAddr>().unwrap().to_string());
    assert_eq!(ip6.to_string(), SocketAddr::from(ip6).to_string());
}

#[test]
#[cfg(unix)]
fn test_socket_addr_unix() {
    assert_eq!("unix:/tmp/sock".parse::<SocketAddr>().unwrap().to_string(), "unix:/tmp/sock");
    assert!("/tmp/sock".parse::<SocketAddr>().is_err());
}
