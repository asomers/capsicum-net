use capsicum_net::CasperExt;

use crate::CASPER;

mod bind {
    use super::*;

    #[tokio::test]
    async fn eafnosupport() {
        use capsicum_net::tokio::TcpSocketExt;
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let want = "127.0.0.1:8083".parse().unwrap();
        let socket = tokio::net::TcpSocket::new_v6().unwrap();
        let err = socket.cap_bind(&mut cap_net, want).unwrap_err();
        assert_eq!(err.raw_os_error(), Some(libc::EAFNOSUPPORT));
    }

    #[tokio::test]
    async fn ok() {
        use capsicum_net::tokio::TcpSocketExt;
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let want = "127.0.0.1:8083".parse().unwrap();
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        socket.cap_bind(&mut cap_net, want).unwrap();
        let bound = socket.local_addr().unwrap();
        assert_eq!(want, bound);
    }
}
