// vim: tw=80
#![cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
use std::{io, os::fd::AsRawFd};

use tokio::net::TcpSocket;

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
