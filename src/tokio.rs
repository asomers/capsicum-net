use std::{io, os::fd::AsRawFd};

use nix::sys::socket::{SockaddrIn, SockaddrIn6, SockaddrLike};

use super::{CapNetAgent, ffi};

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
    ///     let casper = unsafe { Casper::new().unwrap() };
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
    fn cap_bind(&self, agent: &mut CapNetAgent, addr: std::net::SocketAddr) -> io::Result<()>;
}

impl TcpSocketExt for tokio::net::TcpSocket {
    fn cap_bind(&self, agent: &mut CapNetAgent, addr: std::net::SocketAddr) -> io::Result<()>
    {
        let ap = agent.0.as_mut_ptr();
        let sock = self.as_raw_fd();
        let res = match addr {
            // Even though std::net::SocketAddrV4 is probably stored identically
            // to libc::sockaddr_in, that isn't guaranteed, so we must convert
            // it.  Nix's representation _is_ guaranteed.  Ditto for
            // SocketAddtV6.
            std::net::SocketAddr::V4(addr) => {
                let sin = SockaddrIn::from(addr);
                unsafe {
                    ffi::cap_bind(ap, sock, sin.as_ptr(), sin.len())
                }
            }
            std::net::SocketAddr::V6(addr) => {
                let sin6 = SockaddrIn6::from(addr);
                unsafe {
                    ffi::cap_bind(ap, sock, sin6.as_ptr(), sin6.len())
                }
            }
        };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

