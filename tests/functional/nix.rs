// vim: tw=80
use std::os::fd::AsRawFd;

use capsicum_net::CasperExt;
use nix::{
    sys::socket::{
        getsockname,
        socket,
        AddressFamily,
        SockFlag,
        SockType,
        SockaddrIn,
        SockaddrIn6,
        UnixAddr,
    },
    Error,
};
use tempfile::TempDir;

use crate::CASPER;

/// Get a process-wide unique local IPv4 address.
fn get_local_in() -> SockaddrIn {
    SockaddrIn::new(127, 0, 0, 1, crate::next_port())
}

/// Get a process-wide unique local IPv6 address.
fn get_local_in6() -> SockaddrIn6 {
    std::net::SocketAddrV6::new(
        std::net::Ipv6Addr::LOCALHOST,
        crate::next_port(),
        0,
        0,
    )
    .into()
}

mod bind {
    use super::*;

    #[test]
    fn eafnosupport() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let s = socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let want = get_local_in();
        let err = cap_net.bind(&s, &want).unwrap_err();
        assert_eq!(err, Error::EAFNOSUPPORT);
    }

    #[test]
    fn ipv4() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let s = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let want = get_local_in();
        cap_net.bind(&s, &want).unwrap();
        let bound: SockaddrIn = getsockname(s.as_raw_fd()).unwrap();
        assert_eq!(want, bound);
    }

    #[test]
    fn ipv6() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let s = socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let want = get_local_in6();
        cap_net.bind(&s, &want).unwrap();
        let bound: SockaddrIn6 = getsockname(s.as_raw_fd()).unwrap();
        assert_eq!(want, bound);
    }

    #[test]
    fn unix() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let s = socket(
            AddressFamily::Unix,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sock");
        let want = UnixAddr::new(&path).unwrap();
        cap_net.bind(&s, &want).unwrap();
        let bound: UnixAddr = getsockname(s.as_raw_fd()).unwrap();
        assert_eq!(want, bound);
    }
}

mod limit_bind {
    use super::*;

    #[test]
    fn ipv4_negative() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();
        let limit_to = get_local_in();
        let want = get_local_in();
        let mut limit = cap_net.limit();
        limit.bind(&limit_to);
        limit.limit().unwrap();

        let s = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let e = cap_net.bind(&s, &want).unwrap_err();
        assert_eq!(Error::ENOTCAPABLE, e);
    }

    #[test]
    fn ipv4_postive() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();
        let want = get_local_in();
        let mut limit = cap_net.limit();
        limit.bind(&want);
        limit.limit().unwrap();

        let s = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        cap_net.bind(&s, &want).unwrap();
        let bound: SockaddrIn = getsockname(s.as_raw_fd()).unwrap();
        assert_eq!(want, bound);
    }
}
