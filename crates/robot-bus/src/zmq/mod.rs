//! The libzmq operations used by the runtime and broker.

mod ffi;

use std::ffi::{CStr, CString, c_int, c_long, c_short, c_void};
use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
use std::sync::Arc;

pub const DONTWAIT: c_int = 1;
pub const POLLIN: c_short = 1;

/// An error reported by the ZeroMQ transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error(c_int);

impl Error {
    pub const EAGAIN: Self = Self(libc::EAGAIN);
    pub const EFAULT: Self = Self(libc::EFAULT);
    pub(crate) const EINTR: Self = Self(libc::EINTR);
    const EINVAL: Self = Self(libc::EINVAL);

    /// Return the errno reported by libzmq.
    pub fn raw_code(self) -> i32 {
        self.0
    }

    fn last() -> Self {
        // SAFETY: This reads the calling thread's libzmq errno.
        Self(unsafe { ffi::zmq_errno() })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SAFETY: libzmq returns a non-null, NUL-terminated error string.
        let text = unsafe { CStr::from_ptr(ffi::zmq_strerror(self.0)) };
        write!(f, "{}", text.to_string_lossy())
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

fn check(result: c_int) -> Result<c_int> {
    if result == -1 {
        Err(Error::last())
    } else {
        Ok(result)
    }
}

#[derive(Clone, Default)]
pub struct Context(Arc<RawContext>);

struct RawContext(NonNull<c_void>);

// SAFETY: libzmq contexts may be shared between threads. The Arc owns the
// context until the last Context and the last Socket have both been dropped.
unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}

impl Default for RawContext {
    fn default() -> Self {
        // SAFETY: There are no arguments. A successful call creates a context.
        let raw = NonNull::new(unsafe { ffi::zmq_ctx_new() })
            .unwrap_or_else(|| panic!("creating ZeroMQ context: {}", Error::last()));
        Self(raw)
    }
}

impl Drop for RawContext {
    fn drop(&mut self) {
        loop {
            // SAFETY: This owns the context and all sockets have been closed.
            let result = check(unsafe { ffi::zmq_ctx_term(self.0.as_ptr()) });
            if result != Err(Error::EINTR) {
                break;
            }
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn socket(&self, kind: SocketType) -> Result<Socket> {
        // SAFETY: self keeps the context alive, and kind is a stable C constant.
        let raw = NonNull::new(unsafe { ffi::zmq_socket(self.0.0.as_ptr(), kind as c_int) })
            .ok_or_else(Error::last)?;
        Ok(Socket {
            raw,
            _context: self.clone(),
        })
    }
}

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum SocketType {
    PAIR = 0,
    PUB = 1,
    SUB = 2,
    REQ = 3,
    REP = 4,
    DEALER = 5,
    ROUTER = 6,
    XPUB = 9,
    XSUB = 10,
}

pub struct Socket {
    raw: NonNull<c_void>,
    _context: Context,
}

// SAFETY: A socket can move to another thread when it is no longer being used
// by the original thread. It is not Sync, so callers must serialize access.
unsafe impl Send for Socket {}

impl Drop for Socket {
    fn drop(&mut self) {
        // SAFETY: The socket is owned here and its context is still alive.
        unsafe { ffi::zmq_close(self.raw.as_ptr()) };
    }
}

impl Socket {
    pub fn bind(&self, endpoint: &str) -> Result<()> {
        let endpoint = CString::new(endpoint).map_err(|_| Error::EINVAL)?;
        // SAFETY: The socket and NUL-terminated endpoint are valid for the call.
        check(unsafe { ffi::zmq_bind(self.raw.as_ptr(), endpoint.as_ptr()) }).map(drop)
    }

    pub fn connect(&self, endpoint: &str) -> Result<()> {
        let endpoint = CString::new(endpoint).map_err(|_| Error::EINVAL)?;
        // SAFETY: The socket and NUL-terminated endpoint are valid for the call.
        check(unsafe { ffi::zmq_connect(self.raw.as_ptr(), endpoint.as_ptr()) }).map(drop)
    }

    pub fn send(&self, bytes: &[u8], flags: c_int) -> Result<()> {
        // SAFETY: The socket is live and zmq_send copies the supplied slice.
        check(unsafe {
            ffi::zmq_send(self.raw.as_ptr(), bytes.as_ptr().cast(), bytes.len(), flags)
        })
        .map(drop)
    }

    pub fn send_multipart<I>(&self, parts: I, flags: c_int) -> Result<()>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let mut parts = parts.into_iter();
        let Some(mut last) = parts.next() else {
            return Ok(());
        };
        for part in parts {
            self.send(last.as_ref(), flags | ffi::SNDMORE)?;
            last = part;
        }
        self.send(last.as_ref(), flags)
    }

    pub fn recv_bytes(&self, flags: c_int) -> Result<Vec<u8>> {
        Ok(self.recv(flags)?.bytes())
    }

    pub fn recv_multipart(&self, flags: c_int) -> Result<Vec<Vec<u8>>> {
        let mut parts = Vec::new();
        loop {
            let mut message = self.recv(flags)?;
            parts.push(message.bytes());
            // SAFETY: message was initialized and contains the received part.
            if unsafe { ffi::zmq_msg_more(message.0.as_ptr()) } == 0 {
                return Ok(parts);
            }
        }
    }

    fn recv(&self, flags: c_int) -> Result<Message> {
        let mut raw = MaybeUninit::uninit();
        // SAFETY: raw has the size and alignment required by zmq_msg_t.
        check(unsafe { ffi::zmq_msg_init(raw.as_mut_ptr()) })?;
        let mut message = Message(raw);
        // SAFETY: The message is initialized and the socket is live.
        check(unsafe { ffi::zmq_msg_recv(message.0.as_mut_ptr(), self.raw.as_ptr(), flags) })?;
        Ok(message)
    }

    fn set_option(&self, option: c_int, bytes: &[u8]) -> Result<()> {
        // SAFETY: The caller supplies the option's native byte representation.
        check(unsafe {
            ffi::zmq_setsockopt(
                self.raw.as_ptr(),
                option,
                bytes.as_ptr().cast(),
                bytes.len(),
            )
        })
        .map(drop)
    }

    fn set_int(&self, option: c_int, value: c_int) -> Result<()> {
        // SAFETY: The integer has the alignment and size required by the option.
        check(unsafe {
            ffi::zmq_setsockopt(
                self.raw.as_ptr(),
                option,
                ptr::from_ref(&value).cast(),
                size_of::<c_int>(),
            )
        })
        .map(drop)
    }

    fn get_int(&self, option: c_int) -> Result<c_int> {
        let mut value: c_int = 0;
        let mut length = size_of::<c_int>();
        // SAFETY: value is an aligned, writable integer of the specified size.
        check(unsafe {
            ffi::zmq_getsockopt(
                self.raw.as_ptr(),
                option,
                ptr::from_mut(&mut value).cast(),
                &mut length,
            )
        })?;
        if length != size_of::<c_int>() {
            return Err(Error::EINVAL);
        }
        Ok(value)
    }

    pub fn set_linger(&self, millis: i32) -> Result<()> {
        self.set_int(ffi::LINGER, millis)
    }

    pub fn set_sndhwm(&self, hwm: i32) -> Result<()> {
        self.set_int(ffi::SNDHWM, hwm)
    }

    pub fn get_sndhwm(&self) -> Result<i32> {
        self.get_int(ffi::SNDHWM)
    }

    pub fn set_rcvhwm(&self, hwm: i32) -> Result<()> {
        self.set_int(ffi::RCVHWM, hwm)
    }

    pub fn get_rcvhwm(&self) -> Result<i32> {
        self.get_int(ffi::RCVHWM)
    }

    pub fn set_rcvtimeo(&self, millis: i32) -> Result<()> {
        self.set_int(ffi::RCVTIMEO, millis)
    }

    #[cfg(test)]
    pub(crate) fn get_rcvtimeo(&self) -> Result<i32> {
        self.get_int(ffi::RCVTIMEO)
    }

    pub fn set_sndtimeo(&self, millis: i32) -> Result<()> {
        self.set_int(ffi::SNDTIMEO, millis)
    }

    #[cfg(test)]
    pub(crate) fn get_sndtimeo(&self) -> Result<i32> {
        self.get_int(ffi::SNDTIMEO)
    }

    pub fn set_immediate(&self, enabled: bool) -> Result<()> {
        self.set_int(ffi::IMMEDIATE, enabled.into())
    }

    pub fn set_xpub_verbose(&self, enabled: bool) -> Result<()> {
        self.set_int(ffi::XPUB_VERBOSE, enabled.into())
    }

    pub fn set_subscribe(&self, topic: &[u8]) -> Result<()> {
        self.set_option(ffi::SUBSCRIBE, topic)
    }

    pub fn set_unsubscribe(&self, topic: &[u8]) -> Result<()> {
        self.set_option(ffi::UNSUBSCRIBE, topic)
    }

    pub fn set_identity(&self, identity: &[u8]) -> Result<()> {
        self.set_option(ffi::ROUTING_ID, identity)
    }

    pub fn get_last_endpoint(&self) -> Result<std::result::Result<String, Vec<u8>>> {
        let mut bytes = [0; 1024];
        let mut length = bytes.len();
        // SAFETY: libzmq receives a writable byte buffer and its capacity.
        check(unsafe {
            ffi::zmq_getsockopt(
                self.raw.as_ptr(),
                ffi::LAST_ENDPOINT,
                bytes.as_mut_ptr().cast(),
                &mut length,
            )
        })?;
        let endpoint = CStr::from_bytes_with_nul(bytes.get(..length).ok_or(Error::EINVAL)?)
            .map_err(|_| Error::EINVAL)?;
        Ok(String::from_utf8(endpoint.to_bytes().to_vec()).map_err(|err| err.into_bytes()))
    }

    pub fn poll(&self, events: c_short, timeout: i64) -> Result<i32> {
        poll(&mut [self.as_poll_item(events)], timeout)
    }

    pub(crate) fn as_poll_item(&self, events: c_short) -> PollItem<'_> {
        PollItem(
            ffi::PollItem {
                socket: self.raw.as_ptr(),
                fd: 0,
                events,
                revents: 0,
            },
            PhantomData,
        )
    }
}

struct Message(MaybeUninit<ffi::Message>);

impl Message {
    fn bytes(&mut self) -> Vec<u8> {
        // SAFETY: The initialized message owns the data pointer for its whole
        // length. Handle an empty message without making a slice from its pointer.
        unsafe {
            let raw = self.0.as_mut_ptr();
            let length = ffi::zmq_msg_size(raw);
            if length == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(ffi::zmq_msg_data(raw).cast::<u8>(), length).to_vec()
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        // SAFETY: This message was successfully initialized and is still owned.
        unsafe { ffi::zmq_msg_close(self.0.as_mut_ptr()) };
    }
}

#[repr(transparent)]
pub(crate) struct PollItem<'a>(ffi::PollItem, PhantomData<&'a Socket>);

impl PollItem<'_> {
    pub(crate) fn is_readable(&self) -> bool {
        self.0.revents & POLLIN != 0
    }
}

pub(crate) fn poll(items: &mut [PollItem<'_>], timeout: i64) -> Result<i32> {
    let count = c_int::try_from(items.len()).map_err(|_| Error::EINVAL)?;
    let timeout = if size_of::<c_long>() == size_of::<i64>() {
        timeout
    } else {
        timeout.clamp(i64::from(i32::MIN), i64::from(i32::MAX))
    };
    // SAFETY: PollItem is transparent over the C structure. Its lifetime keeps
    // every socket alive until the poll returns.
    check(unsafe { ffi::zmq_poll(items.as_mut_ptr().cast(), count, timeout as c_long) })
}

pub fn proxy_steerable(
    frontend: &mut Socket,
    backend: &mut Socket,
    control: &mut Socket,
) -> Result<()> {
    proxy(frontend, backend, None, control)
}

#[cfg(feature = "broker")]
pub(crate) fn proxy_steerable_with_capture(
    frontend: &mut Socket,
    backend: &mut Socket,
    capture: &mut Socket,
    control: &mut Socket,
) -> Result<()> {
    proxy(frontend, backend, Some(capture), control)
}

fn proxy(
    frontend: &mut Socket,
    backend: &mut Socket,
    capture: Option<&mut Socket>,
    control: &mut Socket,
) -> Result<()> {
    let capture = capture.map_or(ptr::null_mut(), |socket| socket.raw.as_ptr());
    // SAFETY: The live sockets are borrowed exclusively for the call. libzmq
    // accepts a null capture socket when capture is disabled.
    check(unsafe {
        ffi::zmq_proxy_steerable(
            frontend.raw.as_ptr(),
            backend.raw.as_ptr(),
            capture,
            control.raw.as_ptr(),
        )
    })
    .map(drop)
}

pub(crate) fn has_ipc() -> bool {
    // SAFETY: The capability is a static, NUL-terminated C string.
    unsafe { ffi::zmq_has(c"ipc".as_ptr()) == 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_libzmq_version() {
        let (mut major, mut minor, mut patch) = (0, 0, 0);
        // SAFETY: These three output pointers refer to distinct writable ints.
        unsafe { ffi::zmq_version(&mut major, &mut minor, &mut patch) };
        assert_eq!((major, minor, patch), (4, 3, 5));
    }

    #[test]
    fn nonblocking_multipart_preserves_empty_and_large_frames() {
        let context = Context::new();
        let recv = context.socket(SocketType::PAIR).unwrap();
        let send = context.socket(SocketType::PAIR).unwrap();
        let endpoint = "inproc://zmq-multipart-test";
        recv.bind(endpoint).unwrap();
        send.connect(endpoint).unwrap();
        drop(context);

        assert_eq!(recv.recv_multipart(DONTWAIT), Err(Error::EAGAIN));
        let payload = vec![0x8a; 128 * 1024];
        send.send_multipart([b"".as_slice(), &payload, b""], DONTWAIT)
            .unwrap();
        assert_eq!(
            recv.recv_multipart(DONTWAIT).unwrap(),
            [vec![], payload, vec![]],
        );
        assert_eq!(recv.recv_multipart(DONTWAIT), Err(Error::EAGAIN));
    }
}
