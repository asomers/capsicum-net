use std::{
    net::{ToSocketAddrs, UdpSocket},
    os::fd::{AsRawFd, RawFd},
    str::FromStr,
    sync::{Mutex, OnceLock}
};

use capsicum::casper::Casper;
use cstr::cstr;
use ctor::ctor;
use nix::{
    Result,
    errno::Errno,
    sys::socket::{AddressFamily, SockaddrIn, SockaddrLike, SockFlag, SockType,
        getsockname, socket}
};

use capsicum_net::{CasperExt};

// CASPER must be static because it cannot be created after the program becomes
// multithreaded.
static CASPER: OnceLock<Casper> = OnceLock::new();


#[test]
fn t2() {
    let casper = CASPER.get().unwrap();
    let mut cap_net1 = casper.net().unwrap();
    let mut cap_net2 = casper.net().unwrap();
}

#[test]
fn nix() {
    let casper = CASPER.get().unwrap();
    let mut cap_net = casper.net().unwrap();

    let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None)
        .unwrap();
    let want = SockaddrIn::from_str("127.0.0.1:8082").unwrap();
    cap_net.bind(s.as_raw_fd(), &want).unwrap(); 
    let bound: SockaddrIn = getsockname(s.as_raw_fd()).unwrap();
    assert_eq!(want, bound);
}

#[tokio::test]
async fn tokio() {
    use capsicum_net::tokio::TcpSocketExt;
    let casper = CASPER.get().unwrap();
    let mut cap_net = casper.net().unwrap();

    let want = "127.0.0.1:8083".parse().unwrap();
    let socket = tokio::net::TcpSocket::new_v4().unwrap();
    socket.cap_bind(&mut cap_net, want).unwrap();
    let bound = socket.local_addr().unwrap();
    assert_eq!(want, bound);
}

// Casper::new() must be called from a single-threaded context, so we
// do it in ctor, because the test harness will create multiple
// threads.
#[ctor]
unsafe fn casper_initialize() {
    // safe because we are single-threaded during #[ctor]
    let casper = unsafe { Casper::new().unwrap() };
    CASPER.set(casper).unwrap();
}
