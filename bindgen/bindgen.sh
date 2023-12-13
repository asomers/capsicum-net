#! /bin/sh

CRATEDIR=`dirname $0`/..

cat > src/ffi.rs << HERE
#![allow(non_camel_case_types)]
use casper_sys::cap_channel_t;
use libc::sockaddr;
HERE

bindgen --allowlist-function 'cap_bind' \
	--allowlist-function 'cap_net_limit_init' \
	--allowlist-function 'cap_net_limit_bind' \
	--allowlist-function 'cap_net_limit' \
	--allowlist-item '.*CAPNET_BIND.*' \
	--opaque-type 'cap_net_limit_t' \
	--blocklist-type 'cap_channel' \
	--blocklist-type 'cap_channel_t' \
	--blocklist-type 'sockaddr' \
	--blocklist-type 'sa_family_t' \
	${CRATEDIR}/bindgen/wrapper.h >> ${CRATEDIR}/src/ffi.rs

