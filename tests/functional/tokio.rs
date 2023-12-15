// vim: tw=80
use std::os::fd::AsRawFd;

use capsicum_net::{
    tokio::{TcpSocketExt, UdpSocketExt, UnixDatagramExt},
    CasperExt,
};
use tempfile::TempDir;

use crate::{
    std::{get_local_in, get_local_in6},
    CASPER,
};

mod bind {
    use super::*;

    mod tcp {
        use super::*;

        #[tokio::test]
        async fn eafnosupport() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = get_local_in();
            let socket = tokio::net::TcpSocket::new_v6().unwrap();
            let err = socket.cap_bind(&mut cap_net, want).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(libc::EAFNOSUPPORT));
        }

        #[tokio::test]
        async fn ipv4() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = get_local_in();
            let socket = tokio::net::TcpSocket::new_v4().unwrap();
            socket.cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }

        #[tokio::test]
        async fn ipv6() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = get_local_in6();
            let socket = tokio::net::TcpSocket::new_v6().unwrap();
            socket.cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }
    }

    mod udp {
        use super::*;

        #[tokio::test]
        async fn ipv4() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in();
            let socket =
                UdpSocketExt::cap_bind(&mut cap_net, want).await.unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }

        #[tokio::test]
        async fn ipv6() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let want = get_local_in6();
            let socket =
                UdpSocketExt::cap_bind(&mut cap_net, want).await.unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }
    }

    mod unix {
        use super::*;

        #[tokio::test]
        async fn datagram() {
            let mut cap_net = {
                let mut casper = CASPER.get().unwrap().lock().unwrap();
                casper.net().unwrap()
            };

            let dir = TempDir::new().unwrap();
            let path = dir.path().join("sock");
            let socket =
                UnixDatagramExt::cap_bind(&mut cap_net, &path).unwrap();

            // We can't use UnixDatagram::local_addr due to
            // https://github.com/rust-lang/rust/issues/118925 , so use nix's
            // gethostname instead.
            let bound: nix::sys::socket::UnixAddr =
                nix::sys::socket::getsockname(socket.as_raw_fd()).unwrap();
            assert_eq!(path, bound.path().unwrap());
        }
    }
}
