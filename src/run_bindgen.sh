#!/bin/sh

# Run this to regenerate ll.rs after a C library update.
# This is a tempory measure until bindgen can be worked into the build process

export LD_PRELOAD=/usr/lib/libclang.so
bindgen -builtins \
        -I/usr/lib64/clang/3.5.0/include \
        -I/usr/local/include \
        -lgnunetutil \
        -lgnunetgnsrecord \
        ll.h > ll.rs
cat <<EOF >>ll.rs

pub const GNUNET_NO: ::libc::c_int = 0;
pub const GNUNET_OK: ::libc::c_int = 1;
pub const GNUNET_MESSAGE_TYPE_GNS_LOOKUP: u16 = 500;
pub const GNUNET_MESSAGE_TYPE_GNS_LOOKUP_RESULT: u16 = 501;
pub const GNUNET_MESSAGE_TYPE_IDENTITY_START: u16 = 624;
pub const GNUNET_MESSAGE_TYPE_IDENTITY_RESULT_CODE: u16 = 625;
pub const GNUNET_MESSAGE_TYPE_IDENTITY_UPDATE: u16 = 626;
pub const GNUNET_MESSAGE_TYPE_IDENTITY_GET_DEFAULT: u16 = 627;
pub const GNUNET_MESSAGE_TYPE_IDENTITY_SET_DEFAULT: u16 = 628;
pub const GNUNET_MESSAGE_TYPE_CADET_LOCAL_CONNECT: u16 = 272;
pub const GNUNET_MESSAGE_TYPE_CADET_LOCAL_CHANNEL_CREATE: u16 = 273;
pub const GNUNET_DNSPARSER_MAX_NAME_LENGTH: u16 = 253;

unsafe impl Send for Struct_GNUNET_GNSRECORD_Data {}

EOF

