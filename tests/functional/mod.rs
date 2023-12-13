// vim: tw=80
use ::std::sync::{
    atomic::{AtomicU16, Ordering},
    Mutex,
    OnceLock,
};
use capsicum::casper::Casper;
use capsicum_net::CasperExt;
use ctor::ctor;

mod nix;
mod std;
#[cfg(feature = "tokio")]
mod tokio;

// CASPER must be static because it cannot be created after the program becomes
// multithreaded.
static CASPER: OnceLock<Mutex<Casper>> = OnceLock::new();

/// Not that you'd ever want to, but it should be possible to create multiple
/// instances of the service.
#[test]
fn multiple_instances() {
    let mut casper = CASPER.get().unwrap().lock().unwrap();
    let _cap_net1 = casper.net().unwrap();
    let _cap_net2 = casper.net().unwrap();
}

// Casper::new() must be called from a single-threaded context, so we
// do it in ctor, because the test harness will create multiple
// threads.
#[ctor]
unsafe fn casper_initialize() {
    // safe because we are single-threaded during #[ctor]
    let casper = Mutex::new(unsafe { Casper::new().unwrap() });
    CASPER.set(casper).unwrap();
}

// Use an atomic variable to ensure no two tests within this process try to use
// the same port.
// NB: this causes frequent failures with cargo-nextest.
static PORT: AtomicU16 = AtomicU16::new(8000);

fn next_port() -> u16 {
    PORT.fetch_add(1, Ordering::Relaxed)
}
