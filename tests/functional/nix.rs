// vim: tw=80
use std::os::fd::AsRawFd;

use capsicum_net::{CasperExt, LimitFlags};
use nix::{
    sys::socket::{
        getpeername,
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
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

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
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

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
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

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
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

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

mod limit {
    use super::*;

    mod bind {
        use super::*;

        #[test]
        fn badmode() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };
            let want = get_local_in();
            let mut limit = cap_net.limit(LimitFlags::CONNECT);
            limit.bind(&want);
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
        fn ipv4_excluded() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };
            let limit_to = get_local_in();
            let want = get_local_in();
            let mut limit = cap_net.limit(LimitFlags::BIND);
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
        fn ipv4_included() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };
            let want = get_local_in();
            let mut limit = cap_net.limit(LimitFlags::BIND);
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

    mod connect {
        use super::*;

        #[test]
        fn badmode() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };
            let want = get_local_in();
            let mut limit = cap_net.limit(LimitFlags::BIND);
            limit.connect(&want);
            limit.limit().unwrap();

            let _server_sock =
                std::net::TcpListener::bind(std::net::SocketAddrV4::from(want));

            let client_sock = socket(
                AddressFamily::Inet,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .unwrap();
            let e = cap_net.connect(&client_sock, &want).unwrap_err();
            assert_eq!(Error::ENOTCAPABLE, e);
        }

        #[test]
        fn ipv4_excluded() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let limit_to = get_local_in();
            let _server_sock =
                std::net::TcpListener::bind(std::net::SocketAddrV4::from(want));

            let mut limit = cap_net.limit(LimitFlags::CONNECT);
            limit.connect(&limit_to);
            limit.limit().unwrap();

            let client_sock = socket(
                AddressFamily::Inet,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .unwrap();
            let e = cap_net.connect(&client_sock, &want).unwrap_err();
            assert_eq!(Error::ENOTCAPABLE, e);
        }

        #[test]
        fn ipv4_included() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let _server_sock =
                std::net::TcpListener::bind(std::net::SocketAddrV4::from(want));

            let client_sock = socket(
                AddressFamily::Inet,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .unwrap();
            cap_net.connect(&client_sock, &want).unwrap();
            let peer = getpeername(client_sock.as_raw_fd()).unwrap();
            assert_eq!(want, peer);
        }
    }
}

mod connect {
    use super::*;

    #[test]
    fn eaddrnotavail() {
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

        // 127.100.0.1 is reserved for local use, but most likely not assigned
        let want = SockaddrIn::new(127, 100, 0, 1, crate::next_port());

        let client_sock = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        let e = cap_net.connect(&client_sock, &want).unwrap_err();
        assert_eq!(Error::EADDRNOTAVAIL, e);
    }

    #[test]
    fn ipv4() {
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

        let want = get_local_in();
        let _server_sock =
            std::net::TcpListener::bind(std::net::SocketAddrV4::from(want));

        let client_sock = socket(
            AddressFamily::Inet,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        cap_net.connect(&client_sock, &want).unwrap();
        let peer = getpeername(client_sock.as_raw_fd()).unwrap();
        assert_eq!(want, peer);
    }

    #[test]
    fn ipv6() {
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

        let want = get_local_in6();
        let _server_sock =
            std::net::TcpListener::bind(std::net::SocketAddrV6::from(want));

        let client_sock = socket(
            AddressFamily::Inet6,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        cap_net.connect(&client_sock, &want).unwrap();
        let peer = getpeername(client_sock.as_raw_fd()).unwrap();
        assert_eq!(want, peer);
    }

    #[test]
    fn unix() {
        let mut cap_net = {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            casper.net().unwrap()
        };

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("sock");
        let want = UnixAddr::new(&path).unwrap();
        let _server_sock = std::os::unix::net::UnixListener::bind(&path);

        let client_sock = socket(
            AddressFamily::Unix,
            SockType::Stream,
            SockFlag::empty(),
            None,
        )
        .unwrap();
        cap_net.connect(&client_sock, &want).unwrap();
        let peer = getpeername(client_sock.as_raw_fd()).unwrap();
        assert_eq!(want, peer);
    }
}
