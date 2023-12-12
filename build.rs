// vim: tw=80

fn main() {
    use std::{env, path::PathBuf};

    let bindings = bindgen::Builder::default()
        .header_contents("wrapper.h", "#define WITH_CASPER")
        .header("/usr/include/sys/nv.h")
        .header("/usr/include/libcasper.h")
        .header("/usr/include/casper/cap_net.h")
        .allowlist_function("cap_bind")
        .allowlist_function("cap_net_limit_init")
        .allowlist_function("cap_net_limit_bind")
        .allowlist_function("cap_net_limit")
        .allowlist_item("CAPNET_BIND")
        .opaque_type("cap_net_limit_t")
        .blocklist_type("cap_channel")
        .blocklist_type("cap_channel_t")
        .blocklist_type("sockaddr")
        .blocklist_type("sa_family_t")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    println!("cargo:rustc-link-lib=cap_net")
}
