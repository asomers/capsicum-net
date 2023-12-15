// vim: tw=80
//! Extension traits for use with Tokio's socket types

#![cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
use std::{io, net::ToSocketAddrs, os::fd::AsFd, path::Path};

use tokio::net::{TcpSocket, UdpSocket, UnixDatagram};

use super::CapNetAgent;

/// Adds extra features to `tokio::net::TcpSocket` that require Casper.
pub trait TcpSocketExt {
    /// Bind a `tokio::net::TcpSocket` to a port.
    ///
    /// # Examples
    /// ```
    /// use std::{io, str::FromStr };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, tokio::TcpSocketExt};
    /// use tokio::net::TcpSocket;
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() -> io::Result<()> {
    ///     // Safe because we are single-threaded
    ///     let mut casper = unsafe { Casper::new().unwrap() };
    ///     let mut cap_net = casper.net().unwrap();
    ///
    ///     let addr = "127.0.0.1:8080".parse().unwrap();
    ///     let socket = TcpSocket::new_v4()?;
    ///     socket.cap_bind(&mut cap_net, addr)?;
    ///
    ///     let listener = socket.listen(1024)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn cap_bind(
        &self,
        agent: &mut CapNetAgent,
        addr: std::net::SocketAddr,
    ) -> io::Result<()>;
}

impl TcpSocketExt for TcpSocket {
    fn cap_bind(
        &self,
        agent: &mut CapNetAgent,
        addr: std::net::SocketAddr,
    ) -> io::Result<()> {
        let sock = self.as_fd();
        agent.bind_std_fd(sock, addr)
    }
}

/// Extension "trait" for `tokio::net::UdpSocket`.
///
/// It functions like an extension trait with only static methods, though it's
/// technically a struct.
pub struct UdpSocketExt {}

impl UdpSocketExt {
    /// Bind a `tokio::net::UdpSocket` to a port.
    ///
    /// # Examples
    /// ```
    /// use std::{io, str::FromStr };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, tokio::UdpSocketExt};
    /// use tokio::net::UdpSocket;
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() -> io::Result<()> {
    ///     // Safe because we are single-threaded
    ///     let mut casper = unsafe { Casper::new().unwrap() };
    ///     let mut cap_net = casper.net().unwrap();
    ///
    ///     let addr = "127.0.0.1:8082";
    ///     let socket = UdpSocketExt::cap_bind(&mut cap_net, addr).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    // This function takes std::net::ToSocketAddrs instead of
    // tokio::net::ToSocketAddrs because the latter has no publicly available
    // methods.
    pub async fn cap_bind<A: ToSocketAddrs>(
        agent: &mut CapNetAgent,
        addrs: A,
    ) -> io::Result<UdpSocket> {
        let std_sock = <std::net::UdpSocket as crate::UdpSocketExt>::cap_bind(
            agent, addrs,
        )?;
        std_sock.set_nonblocking(true)?;
        UdpSocket::from_std(std_sock)
    }
}

/// Extension "trait" for `tokio::net::UnixDatagram`.
///
/// It functions like an extension trait with only static methods, though it's
/// technically a struct.
pub struct UnixDatagramExt {}
impl UnixDatagramExt {
    /// Bind a `tokio::net::UnixDatagram` to a port.
    ///
    /// # Examples
    /// ```no_run
    /// use std::{io, str::FromStr };
    ///
    /// use capsicum::casper::Casper;
    /// use capsicum_net::{CasperExt, tokio::UnixDatagramExt};
    /// use tokio::net::UnixDatagram;
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() -> io::Result<()> {
    ///     // Safe because we are single-threaded
    ///     let mut casper = unsafe { Casper::new().unwrap() };
    ///     let mut cap_net = casper.net().unwrap();
    ///
    ///     let path = "/var/run/foo.sock";
    ///     let socket = UnixDatagramExt::cap_bind(&mut cap_net, path)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn cap_bind<P>(
        agent: &mut CapNetAgent,
        path: P,
    ) -> io::Result<UnixDatagram>
    where
        P: AsRef<Path>,
    {
        let std_sock = <std::os::unix::net::UnixDatagram as crate::UnixDatagramExt>::cap_bind(
            agent, path,
        )?;
        std_sock.set_nonblocking(true)?;
        UnixDatagram::from_std(std_sock)
    }
}
