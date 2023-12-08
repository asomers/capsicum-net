// vim: tw=80
#![cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
use std::{io, net::ToSocketAddrs, os::fd::AsRawFd};

use tokio::net::{TcpSocket, UdpSocket};

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
        let sock = self.as_raw_fd();
        agent.bind_raw_std(sock, addr)
    }
}

/// Extension "trait" for `tokio::net::UdpSocket`.
///
/// It functions like an extension trait with only static methods, though it's
/// technically a struct.
pub struct UdpSocketExt {}

impl UdpSocketExt {
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
