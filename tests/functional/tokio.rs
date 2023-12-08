// vim: tw=80
use capsicum_net::{tokio::TcpSocketExt, CasperExt};

use crate::CASPER;

mod bind {
    use super::*;

    mod tcp {
        use super::*;

        #[tokio::test]
        async fn eafnosupport() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = "127.0.0.1:8083".parse().unwrap();
            let socket = tokio::net::TcpSocket::new_v6().unwrap();
            let err = socket.cap_bind(&mut cap_net, want).unwrap_err();
            assert_eq!(err.raw_os_error(), Some(libc::EAFNOSUPPORT));
        }

        #[tokio::test]
        async fn ipv4() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = "127.0.0.1:8083".parse().unwrap();
            let socket = tokio::net::TcpSocket::new_v4().unwrap();
            socket.cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }

        #[tokio::test]
        async fn ipv6() {
            let mut casper = CASPER.get().unwrap().lock().unwrap();
            let mut cap_net = casper.net().unwrap();

            let want = "[::1]:8088".parse().unwrap();
            let socket = tokio::net::TcpSocket::new_v6().unwrap();
            socket.cap_bind(&mut cap_net, want).unwrap();
            let bound = socket.local_addr().unwrap();
            assert_eq!(want, bound);
        }
    }
}
