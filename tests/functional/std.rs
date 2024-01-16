// vim: tw=80
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::AsRawFd,
};

use capsicum_net::CasperExt;
use tempfile::TempDir;

use crate::CASPER;

/// Get a process-wide unique local IPv4 address.
pub fn get_local_in() -> SocketAddr {
    SocketAddrV4::new(Ipv4Addr::LOCALHOST, crate::next_port()).into()
}

/// Get a process-wide unique local IPv6 address.
pub fn get_local_in6() -> SocketAddr {
    SocketAddrV6::new(Ipv6Addr::LOCALHOST, crate::next_port(), 0, 0).into()
}

mod tcp_listener {
    use std::net::TcpListener;

    use capsicum_net::std::TcpListenerExt;

    use super::*;

    mod bind {
        use super::*;

        #[test]
        fn eaddrinuse() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let _socket1 = TcpListener::cap_bind(&mut cap_net, want).unwrap();
            let err = TcpListener::cap_bind(&mut cap_net, want).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(libc::EADDRINUSE));
        }

        #[test]
        fn no_addresses() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let addrs: Vec<SocketAddr> = Vec::new();
            let err =
                TcpListener::cap_bind(&mut cap_net, &addrs[..]).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        }

        #[test]
        fn ipv4() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let socket = TcpListener::cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }

        #[test]
        fn ipv6() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in6();
            let socket = TcpListener::cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }
    }
}

mod tcp_stream {
    use std::net::{TcpListener, TcpStream};

    use capsicum_net::std::TcpStreamExt;

    use super::*;

    mod connect {
        use super::*;

        #[test]
        fn eaddrnotavail() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            // 127.100.0.1 is reserved for local use, but most likely not assigned
            let want: SocketAddr = SocketAddrV4::new(
                Ipv4Addr::new(127, 100, 0, 1),
                crate::next_port(),
            )
            .into();
            let err = TcpStream::cap_connect(&mut cap_net, want).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(libc::EADDRNOTAVAIL));
        }

        #[test]
        fn ipv4() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let _server_socket = TcpListener::bind(want).unwrap();
            let client_socket =
                TcpStream::cap_connect(&mut cap_net, want).unwrap();
            let connected = client_socket.peer_addr().unwrap();
            assert_eq!(want, connected);
        }

        #[test]
        fn ipv6() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in6();
            let _server_socket = TcpListener::bind(want).unwrap();
            let client_socket =
                TcpStream::cap_connect(&mut cap_net, want).unwrap();
            let connected = client_socket.peer_addr().unwrap();
            assert_eq!(want, connected);
        }
    }
}

mod udp_socket {
    use std::net::UdpSocket;

    use capsicum_net::std::UdpSocketExt;

    use super::*;

    mod bind {
        use super::*;

        #[test]
        fn eaddrinuse() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let _socket1 = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
            let err = UdpSocket::cap_bind(&mut cap_net, want).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(libc::EADDRINUSE));
        }

        #[test]
        fn no_addresses() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let addrs: Vec<SocketAddr> = Vec::new();
            let err =
                UdpSocket::cap_bind(&mut cap_net, &addrs[..]).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        }

        #[test]
        fn ipv4() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let socket = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }

        #[test]
        fn ipv6() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in6();
            let socket = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }
    }

    mod connect {
        use super::*;

        #[test]
        fn ipv4() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
            socket.cap_connect(&mut cap_net, want).unwrap();
            let connected = socket.peer_addr().unwrap();
            assert_eq!(want, connected);
        }

        #[test]
        fn ipv6() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in6();
            let socket = UdpSocket::bind("[::0]:0").unwrap();
            socket.cap_connect(&mut cap_net, want).unwrap();
            let connected = socket.peer_addr().unwrap();
            assert_eq!(want, connected);
        }
    }
}

mod unix_datagram {
    use std::os::unix::net::UnixDatagram;

    use capsicum_net::std::UnixDatagramExt;

    use super::*;

    mod bind {
        use super::*;

        #[test]
        fn ok() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let dir = TempDir::new().unwrap();
            let path = dir.path().join("sock");
            let socket = UnixDatagram::cap_bind(&mut cap_net, &path).unwrap();

            // We can't use UnixDatagram::local_addr due to
            // https://github.com/rust-lang/rust/issues/118925 , so use nix's
            // gethostname instead.
            let bound: nix::sys::socket::UnixAddr =
                nix::sys::socket::getsockname(socket.as_raw_fd()).unwrap();
            assert_eq!(path, bound.path().unwrap());
        }
    }
}

mod unix_listener {
    use std::os::unix::net::UnixListener;

    use capsicum_net::std::UnixListenerExt;

    use super::*;

    mod bind {
        use super::*;

        #[test]
        fn ok() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let dir = TempDir::new().unwrap();
            let path = dir.path().join("sock");
            let socket = UnixListener::cap_bind(&mut cap_net, &path).unwrap();

            // We can't use UnixListener::local_addr due to
            // https://github.com/rust-lang/rust/issues/118925 , so use nix's
            // gethostname instead.
            let bound: nix::sys::socket::UnixAddr =
                nix::sys::socket::getsockname(socket.as_raw_fd()).unwrap();
            assert_eq!(path, bound.path().unwrap());
        }
    }
}
