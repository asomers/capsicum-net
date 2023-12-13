// vim: tw=80
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
//! the agent, this library has three interfaces:
//!
//! * Low-level methods directly on the `CapNetAgent` object.  These work well
//! with the [nix](https://docs.rs/nix/0.27.1/nix/) crate.
//! * Extension traits that work on the standard socket types, like
//! [`UdpSocketExt`].
//! * Extension traits that work with tokio types, like
//! [`TcpSocketExt`](tokio::TcpSocketExt).
//!
//! # Example
//! In this example, we create a new UdpSocket and bind it to a port.  Such a
//! thing is normally not allowed in capability mode, but cap_bind lets us do
//! it.
//!
//! ```
//! use std::{io, str::FromStr, net::UdpSocket };
//!
//! use capsicum::casper::Casper;
//! use capsicum_net::{CasperExt, UdpSocketExt};
//!
//! // Safe because we are single-threaded
//! let mut casper = unsafe { Casper::new().unwrap() };
//! let mut cap_net = casper.net().unwrap();
//!
//! capsicum::enter();
//!
//! // At this point regular bind(2) will fail because we're in capability mode.
//! UdpSocket::bind("127.0.0.1:8086").unwrap_err();
//!
//! // But cap_bind will still suceed.
//! let socket = UdpSocket::cap_bind(&mut cap_net, "127.0.0.1:8086")
//!     .unwrap();
//! ```
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
use std::{
    io,
    marker::PhantomData,
    net::{ToSocketAddrs, UdpSocket},
    os::fd::{AsRawFd, RawFd},
};

use capsicum::casper;
use cstr::cstr;
use nix::{
    errno::Errno,
    sys::socket::{
        AddressFamily,
        SockFlag,
        SockType,
        SockaddrIn,
        SockaddrIn6,
        SockaddrLike,
    },
    Result,
};

mod ffi;

#[cfg(feature = "tokio")]
pub mod tokio;

casper::service_connection! {
    /// A connection to the Casper
    /// [cap_net(3)](https://man.freebsd.org/cgi/man.cgi?query=cap_net) service.
    #[derive(Debug)]
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
    /// use nix::sys::socket::{
    ///     AddressFamily, SockaddrIn, SockaddrLike, SockFlag,
    ///     SockType, socket
    /// };
    ///
    /// // Safe if we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    /// let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(),
    ///     None).unwrap();
    /// let addr = SockaddrIn::from_str("127.0.0.1:8081").unwrap();
    /// cap_net.bind(s.as_raw_fd(), &addr).unwrap();
    /// ```
    pub fn bind(&mut self, sock: RawFd, addr: &dyn SockaddrLike) -> Result<()> {
        let res = unsafe {
            ffi::cap_bind(self.0.as_mut_ptr(), sock, addr.as_ptr(), addr.len())
        };
        Errno::result(res).map(drop)
    }

    /// Helper that binds a raw socket to a std sockaddr
    fn bind_raw_std(
        &mut self,
        sock: RawFd,
        addr: std::net::SocketAddr,
    ) -> io::Result<()> {
        let ap = self.0.as_mut_ptr();
        let res = match addr {
            // Even though std::net::SocketAddrV4 is probably stored identically
            // to libc::sockaddr_in, that isn't guaranteed, so we must convert
            // it.  Nix's representation _is_ guaranteed.  Ditto for
            // SocketAddrV6.
            // XXX ffi::cap_bind is technically a blocking operation.  It blocks
            // within the C library.  But the communication is always local, and
            // in cursory testing is < 0.2 ms, so we'll do it in an ordinary
            // tokio thread.
            std::net::SocketAddr::V4(addr) => {
                let sin = SockaddrIn::from(addr);
                unsafe { ffi::cap_bind(ap, sock, sin.as_ptr(), sin.len()) }
            }
            std::net::SocketAddr::V6(addr) => {
                let sin6 = SockaddrIn6::from(addr);
                unsafe { ffi::cap_bind(ap, sock, sin6.as_ptr(), sin6.len()) }
            }
        };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    /// Return an opaque handle used to further limit the capabilities of the
    /// `cap_net` service.
    ///
    /// Each time a [`Limit`] is constructed and applied it can reduce, but
    /// never enlarge, the service's capabilities.
    ///
    /// # Example
    /// ```
    /// use std::{
    ///     os::fd::AsRawFd,
    ///     str::FromStr
    /// };
    /// use capsicum::casper::Casper;
    /// use capsicum_net::CasperExt;
    /// use nix::sys::socket::{SockaddrIn, SockaddrLike};
    ///
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    /// let mut limit = cap_net.limit();
    /// let addr = SockaddrIn::from_str("127.0.0.1:8083").unwrap();
    /// limit.bind(&addr);
    /// limit.limit();
    /// // Now the service will refuse attempts to bind to any other address or
    /// // port.
    /// ```
    pub fn limit(&mut self) -> Limit {
        // NB: in the future, when capsicum-net supports more operations, the
        // mode will be user-supplied.
        let mode: u64 = ffi::CAPNET_BIND.into();
        let limit =
            unsafe { ffi::cap_net_limit_init(self.0.as_mut_ptr(), mode) };
        assert!(!limit.is_null());
        Limit {
            limit,
            phantom: PhantomData,
        }
    }
}

/// Used to limit which operations will be allowed by the [`CapNetAgent`].
#[repr(transparent)]
pub struct Limit<'a> {
    limit:   *mut ffi::cap_net_limit_t,
    // Because cap_net_limit_t stores a pointer to cap_channel_t
    phantom: PhantomData<&'a mut CapNetAgent>,
}

impl<'a> Limit<'a> {
    /// Limit the `cap_net` service to only allow binding to the given address.
    ///
    /// May be called multiple times to allow binding to multiple addresses.
    pub fn bind(&mut self, sa: &dyn SockaddrLike) -> &mut Self {
        let newlimit = unsafe {
            ffi::cap_net_limit_bind(self.limit, sa.as_ptr(), sa.len())
        };
        assert_eq!(newlimit, self.limit);
        self
    }

    /// Actually apply the limits
    pub fn limit(self) -> io::Result<()> {
        let res = unsafe { ffi::cap_net_limit(self.limit) };
        if res == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
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
    /// use capsicum_net::{CasperExt, UdpSocketExt};
    ///
    /// // Safe because we are single-threaded
    /// let mut casper = unsafe { Casper::new().unwrap() };
    /// let mut cap_net = casper.net().unwrap();
    ///
    /// let socket = UdpSocket::cap_bind(&mut cap_net, "127.0.0.1:8086")
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
        let mut last_err = None;
        for addr in addrs.to_socket_addrs()? {
            let family = if addr.is_ipv4() {
                AddressFamily::Inet
            } else {
                AddressFamily::Inet6
            };
            let sock = nix::sys::socket::socket(
                family,
                SockType::Datagram,
                SockFlag::empty(),
                None,
            )
            .map_err(std::io::Error::from)?;
            match agent.bind_raw_std(sock.as_raw_fd(), addr) {
                Ok(()) => return Ok(std::net::UdpSocket::from(sock)),
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
