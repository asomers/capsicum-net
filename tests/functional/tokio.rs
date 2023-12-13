// vim: tw=80
use capsicum_net::{
    tokio::{TcpSocketExt, UdpSocketExt},
    CasperExt,
};

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
}
