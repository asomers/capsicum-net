// vim: tw=80
use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use capsicum_net::{CasperExt, UdpSocketExt};

use crate::CASPER;

mod bind {
    use super::*;

    #[test]
    fn eaddrinuse() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let want: std::net::SocketAddr = "127.0.0.1:8085".parse().unwrap();
        let _socket1 = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
        let err = UdpSocket::cap_bind(&mut cap_net, want).unwrap_err();
        assert_eq!(err.raw_os_error(), Some(libc::EADDRINUSE));
    }

    #[test]
    fn no_addresses() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let addrs: Vec<SocketAddr> = Vec::new();
        let err = UdpSocket::cap_bind(&mut cap_net, &addrs[..]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn ipv4() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let want: std::net::SocketAddr = "127.0.0.1:8084".parse().unwrap();
        let socket = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
        let bound = socket.local_addr().unwrap();
        assert_eq!(want, bound);
    }

    #[test]
    fn ipv6() {
        let mut casper = CASPER.get().unwrap().lock().unwrap();
        let mut cap_net = casper.net().unwrap();

        let want: std::net::SocketAddr = "[::1]:8087".parse().unwrap();
        let socket = UdpSocket::cap_bind(&mut cap_net, want).unwrap();
        let bound = socket.local_addr().unwrap();
        assert_eq!(want, bound);
    }
}
