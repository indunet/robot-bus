//! Stable C declarations from libzmq 4.3.5's `include/zmq.h`.

use std::ffi::{c_char, c_int, c_long, c_short, c_void};
use std::mem::MaybeUninit;

pub(super) const SUBSCRIBE: c_int = 6;
pub(super) const UNSUBSCRIBE: c_int = 7;
pub(super) const ROUTING_ID: c_int = 5;
pub(super) const LINGER: c_int = 17;
pub(super) const SNDHWM: c_int = 23;
pub(super) const RCVHWM: c_int = 24;
pub(super) const RCVTIMEO: c_int = 27;
pub(super) const SNDTIMEO: c_int = 28;
pub(super) const LAST_ENDPOINT: c_int = 32;
pub(super) const IMMEDIATE: c_int = 39;
pub(super) const XPUB_VERBOSE: c_int = 40;
pub(super) const SNDMORE: c_int = 2;

// zmq_msg_t has 64 opaque bytes aligned to the pointer size on our targets.
#[repr(C)]
pub(super) struct Message([MaybeUninit<usize>; 64 / size_of::<usize>()]);

#[cfg(not(windows))]
type FileDescriptor = c_int;
#[cfg(windows)]
type FileDescriptor = usize;

#[repr(C)]
pub(super) struct PollItem {
    pub(super) socket: *mut c_void,
    pub(super) fd: FileDescriptor,
    pub(super) events: c_short,
    pub(super) revents: c_short,
}

unsafe extern "C" {
    pub(super) fn zmq_errno() -> c_int;
    pub(super) fn zmq_strerror(error: c_int) -> *const c_char;
    pub(super) fn zmq_has(capability: *const c_char) -> c_int;
    #[cfg(test)]
    pub(super) fn zmq_version(major: *mut c_int, minor: *mut c_int, patch: *mut c_int);

    pub(super) fn zmq_ctx_new() -> *mut c_void;
    pub(super) fn zmq_ctx_term(context: *mut c_void) -> c_int;
    pub(super) fn zmq_socket(context: *mut c_void, kind: c_int) -> *mut c_void;
    pub(super) fn zmq_close(socket: *mut c_void) -> c_int;
    pub(super) fn zmq_bind(socket: *mut c_void, endpoint: *const c_char) -> c_int;
    pub(super) fn zmq_connect(socket: *mut c_void, endpoint: *const c_char) -> c_int;
    pub(super) fn zmq_setsockopt(
        socket: *mut c_void,
        option: c_int,
        value: *const c_void,
        length: usize,
    ) -> c_int;
    pub(super) fn zmq_getsockopt(
        socket: *mut c_void,
        option: c_int,
        value: *mut c_void,
        length: *mut usize,
    ) -> c_int;

    pub(super) fn zmq_send(
        socket: *mut c_void,
        data: *const c_void,
        length: usize,
        flags: c_int,
    ) -> c_int;
    pub(super) fn zmq_msg_init(message: *mut Message) -> c_int;
    pub(super) fn zmq_msg_close(message: *mut Message) -> c_int;
    pub(super) fn zmq_msg_recv(message: *mut Message, socket: *mut c_void, flags: c_int) -> c_int;
    pub(super) fn zmq_msg_size(message: *const Message) -> usize;
    pub(super) fn zmq_msg_data(message: *mut Message) -> *mut c_void;
    pub(super) fn zmq_msg_more(message: *const Message) -> c_int;

    pub(super) fn zmq_poll(items: *mut PollItem, count: c_int, timeout: c_long) -> c_int;
    pub(super) fn zmq_proxy_steerable(
        frontend: *mut c_void,
        backend: *mut c_void,
        capture: *mut c_void,
        control: *mut c_void,
    ) -> c_int;
}
