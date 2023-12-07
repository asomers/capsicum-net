// vim: tw=80
use std::{os::fd::AsRawFd, str::FromStr};

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
        let want = SockaddrIn::from_str("127.0.0.1:8082").unwrap();
        let err = cap_net.bind(s.as_raw_fd(), &want).unwrap_err();
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
        let want = SockaddrIn::from_str("127.0.0.1:8082").unwrap();
        cap_net.bind(s.as_raw_fd(), &want).unwrap();
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
        let want = SockaddrIn6::from_str("[::1]:8089").unwrap();
        cap_net.bind(s.as_raw_fd(), &want).unwrap();
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
        cap_net.bind(s.as_raw_fd(), &want).unwrap();
        let bound: UnixAddr = getsockname(s.as_raw_fd()).unwrap();
        assert_eq!(want, bound);
    }
}
