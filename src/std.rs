// vim: tw=80
//! Extension traits for socket types from the standard library
use ::std::{
    io,
    net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    os::{
        fd::AsFd,
        unix::net::{UnixDatagram, UnixListener},
    },
    path::Path,
};
use nix::sys::socket::{listen, AddressFamily, Backlog, SockFlag, SockType};

use super::CapNetAgent;

/// Adds extra features to `std::net::TcpListener` that require Casper.
pub trait TcpListenerExt {
    /// Create a new `TcpListener` bound to the specified address.
    ///
    /// # Examples
    /// ```
    /// use std::{io, str::FromStr, net::TcpListener };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::TcpListenerExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let socket = TcpListener::cap_bind(&mut cap_net, "127.0.0.1:8084")
    ///     .unwrap();
    /// ```
    fn cap_bind<A>(
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<TcpListener>
    where
        A: ToSocketAddrs;
}

impl TcpListenerExt for TcpListener {
    fn cap_bind<A>(agent: &mut CapNetAgent, addrs: A) -> io::Result<TcpListener>
    where
        A: ToSocketAddrs,
    {
        let s: TcpListener = agent.bind_std_to_addrs(addrs)?;
        listen(&s, Backlog::MAXALLOWABLE)?;
        Ok(s)
    }
}

/// Adds extra features to `std::net::TcpStream` that require Casper.
pub trait TcpStreamExt {
    /// Open a TCP connection to a remote host, connecting via a `cap_net`
    /// service.
    ///
    /// # Examples
    /// ```no_run
    /// use std::{io, str::FromStr, net::TcpStream };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::TcpStreamExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let sock = TcpStream::cap_connect(&mut cap_net, "8.8.8.8:53").unwrap();
    /// ```
    fn cap_connect<A: ToSocketAddrs>(
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<TcpStream>;
}

impl TcpStreamExt for TcpStream {
    fn cap_connect<A: ToSocketAddrs>(
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<TcpStream> {
        let mut last_err = None;
        for addr in addrs.to_socket_addrs()? {
            let family = if addr.is_ipv4() {
                AddressFamily::Inet
            } else {
                AddressFamily::Inet6
            };
            let sock = nix::sys::socket::socket(
                family,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .map_err(io::Error::from)?;
            match agent.connect_std_fd(sock.as_fd(), addr) {
                Ok(()) => return Ok(TcpStream::from(sock)),
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "could not resolve to any addresses",
            )
        }))
    }
}

/// Adds extra features to `std::net::UdpSocket` that require Casper.
pub trait UdpSocketExt {
    /// Bind a `std::net::UdpSocket` to a port.
    ///
    /// # Examples
    /// ```
    /// use std::{io, str::FromStr, net::UdpSocket };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::UdpSocketExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let socket = UdpSocket::cap_bind(&mut cap_net, "127.0.0.1:8088")
    ///     .unwrap();
    /// ```
    fn cap_bind<A>(agent: &mut CapNetAgent, addr: A) -> io::Result<UdpSocket>
    where
        A: ToSocketAddrs;

    /// Connects this UDP socket to a remote address, using a `cap_net` service.
    ///
    /// # Examples
    /// ```no_run
    /// use std::{io, str::FromStr, net::UdpSocket };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::UdpSocketExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    /// socket.cap_connect(&mut cap_net, "8.8.8.8:53").unwrap();
    /// ```
    fn cap_connect<A>(
        &self,
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<()>
    where
        A: ToSocketAddrs;
}

impl UdpSocketExt for UdpSocket {
    fn cap_bind<A>(agent: &mut CapNetAgent, addrs: A) -> io::Result<UdpSocket>
    where
        A: ToSocketAddrs,
    {
        agent.bind_std_to_addrs(addrs)
    }

    fn cap_connect<A>(
        &self,
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<()>
    where
        A: ToSocketAddrs,
    {
        agent.connect_std_to_addrs(self.as_fd(), addrs)
    }
}

/// Adds extra features to `std::os::unix::net::UnixDatagram` that require
/// Casper.
pub trait UnixDatagramExt {
    /// Bind a `std::net::UdpSocket` to a port.
    ///
    /// # Examples
    /// ```no_run
    /// use std::{io, str::FromStr, os::unix::net::UnixDatagram };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::UnixDatagramExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let path = "/var/run/foo.sock";
    /// let socket = UnixDatagram::cap_bind(&mut cap_net, &path).unwrap();
    /// ```
    fn cap_bind<P>(
        agent: &mut CapNetAgent,
        path: P,
    ) -> io::Result<UnixDatagram>
    where
        P: AsRef<Path>;
}

impl UnixDatagramExt for UnixDatagram {
    fn cap_bind<P>(agent: &mut CapNetAgent, path: P) -> io::Result<UnixDatagram>
    where
        P: AsRef<Path>,
    {
        let s = agent.bind_std_unix(SockType::Datagram, path)?;
        Ok(UnixDatagram::from(s))
    }
}

/// Adds extra features to `std::os::unix::net::UnixListener` that require
/// Casper.
pub trait UnixListenerExt {
    /// Bind a `std::net::UdpSocket` to a port.
    ///
    /// # Examples
    /// ```no_run
    /// use std::{io, str::FromStr, os::unix::net::UnixListener };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, std::UnixListenerExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let path = "/var/run/foo.sock";
    /// let socket = UnixListener::cap_bind(&mut cap_net, &path).unwrap();
    /// ```
    fn cap_bind<P>(
        agent: &mut CapNetAgent,
        path: P,
    ) -> io::Result<UnixListener>
    where
        P: AsRef<Path>;
}

impl UnixListenerExt for UnixListener {
    fn cap_bind<P>(agent: &mut CapNetAgent, path: P) -> io::Result<UnixListener>
    where
        P: AsRef<Path>,
    {
        let s = agent.bind_std_unix(SockType::Stream, path)?;
        listen(&s, Backlog::MAXALLOWABLE)?;
        Ok(UnixListener::from(s))
    }
}
