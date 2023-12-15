// vim: tw=80
//! Extension traits for socket types from the standard library
use super::*;

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
        // -1 means "max value", and it's what the standard library does.  It's
        // a Nix bug that we can't use -1 here.
        // https://github.com/nix-rust/nix/issues/2264
        listen(&s, -1i32 as usize)?;
        Ok(s)
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
}

impl UdpSocketExt for UdpSocket {
    fn cap_bind<A>(agent: &mut CapNetAgent, addrs: A) -> io::Result<UdpSocket>
    where
        A: ToSocketAddrs,
    {
        agent.bind_std_to_addrs(addrs)
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
        // -1 means "max value", and it's what the standard library does.  It's
        // a Nix bug that we can't use -1 here.
        // https://github.com/nix-rust/nix/issues/2264
        listen(&s, -1i32 as usize)?;
        Ok(UnixListener::from(s))
    }
}
