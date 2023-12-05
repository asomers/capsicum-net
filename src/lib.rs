//! Rust bindings to FreeBSD's
//! [cap_net(3)](https://man.freebsd.org/cgi/man.cgi?query=cap_net) library.
//!
//! cap_net allows access to several network APIs that are forbidden in
//! capability mode by delegating them to an unsandboxed process, the Casper
//! daemon.
//!
//! The main entry point for this library is [`CapNetAgent`].  The agent may be
//! created at any time, whether in capability mode or not, as long as the
//! Casper daemon was started prior to entering capability mode.  After creating
//! the agent, this library has two interfaces:
//!
//! * Low-level methods directly on the `CapNetAgent` object.
//! * Extension traits that work with tokio types, like
//! [`TcpSocketExt`](tokio::TcpSocketExt).
use std::{
    io,
    net::{ToSocketAddrs, UdpSocket},
    os::fd::{AsRawFd, RawFd}
};

use capsicum::casper;
use cstr::cstr;
use nix::{
    Result,
    errno::Errno,
    sys::socket::{SockaddrIn, SockaddrIn6, SockaddrLike}
};

#[allow(non_camel_case_types)]
mod ffi {
    use casper_sys::cap_channel_t;
    use libc::sockaddr;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(feature = "tokio")]
pub mod tokio;

casper::service_connection!{
    pub CapNetAgent,
    cstr!("system.net"),
    net
}

impl CapNetAgent {
    /// A low-level bind(2) workalike, but in capability mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     os::fd::AsRawFd,
    ///     str::FromStr
    /// };
    /// use capsicum::casper::Casper;
    /// use capsicum_net::CasperExt;
    /// use nix::{
    ///     Result,
    ///     errno::Errno,
    ///     sys::socket::{
    ///         AddressFamily, SockaddrIn, SockaddrLike, SockFlag,
    ///         SockType, getsockname, socket
    ///     }
    /// };
    ///
    /// // Safe if we are single-threaded
    /// let casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    /// let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(),
    ///     None).unwrap();
    /// let addr = SockaddrIn::from_str("127.0.0.1:8081").unwrap();
    /// cap_net.bind(s.as_raw_fd(), &addr).unwrap(); 
    /// ```
    pub fn bind(&mut self, sock: RawFd, addr: &dyn SockaddrLike) -> Result<()>
    {
        let res = unsafe {
            ffi::cap_bind(self.0.as_mut_ptr(), sock, addr.as_ptr(), addr.len())
        };
        Errno::result(res).map(drop)
    }
}
trait UdpSocketExt {
    fn bind<A>(agent: &mut CapNetAgent, addr: A) -> io::Result<UdpSocket>
        where A: ToSocketAddrs;
}

impl UdpSocketExt for UdpSocket {
    fn bind<A>(agent: &mut CapNetAgent, addrs: A) -> io::Result<UdpSocket>
        where A: ToSocketAddrs
    {
        use nix::{
            Result,
            errno::Errno,
            sys::socket::{
                AddressFamily, SockaddrIn, SockaddrLike, SockFlag,
                SockType, getsockname, socket
            }
        };

        let ap = agent.0.as_mut_ptr();
        let mut last_err = None;
        for addr in addrs.to_socket_addrs()? {
            let (sock, res) = match addr {
                // Even though std::net::SocketAddrV4 is probably stored
                // identically to libc::sockaddr_in, that isn't guaranteed, so
                // we must convert it.  Nix's representation _is_ guaranteed.
                // Ditto for SocketAddtV6.
                std::net::SocketAddr::V4(addr) => {
                    let sock = nix::sys::socket::socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(),
                        None).unwrap(); // TODO: convert error type and return it
                    let sin = SockaddrIn::from(addr);
                    let res = unsafe {
                        ffi::cap_bind(ap, sock.as_raw_fd(), sin.as_ptr(), sin.len())
                    };
                    (sock, res)
                }
                std::net::SocketAddr::V6(addr) => {
                    let sock = nix::sys::socket::socket(AddressFamily::Inet6, SockType::Datagram, SockFlag::empty(),
                        None).unwrap(); // TODO: convert error type and return it
                    let sin6 = SockaddrIn6::from(addr);
                    let res = unsafe {
                        ffi::cap_bind(ap, sock.as_raw_fd(), sin6.as_ptr(), sin6.len())
                    };
                    (sock, res)
                }
            };
            if res == 0 {
                return Ok(std::net::UdpSocket::from(sock))
            } else {
                last_err = Some(std::io::Error::last_os_error());
            }
        }
        Err(last_err.unwrap_or_else(|| {
            todo!()
            //io::const_io_error!(ErrorKind::InvalidInput, "could not resolve to any addresses")
        }))
    }

}
