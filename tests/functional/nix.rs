use std::{
    os::fd::AsRawFd,
    str::FromStr,
};

use capsicum_net::CasperExt;
use nix::sys::socket::{
    AddressFamily, SockaddrIn, SockFlag, SockType, getsockname, socket
};

use crate::CASPER;

#[test]
fn nix() {
    let mut casper = CASPER.get().unwrap().lock().unwrap();
    let mut cap_net = casper.net().unwrap();

    let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None)
        .unwrap();
    let want = SockaddrIn::from_str("127.0.0.1:8082").unwrap();
    cap_net.bind(s.as_raw_fd(), &want).unwrap(); 
    let bound: SockaddrIn = getsockname(s.as_raw_fd()).unwrap();
    assert_eq!(want, bound);
}


