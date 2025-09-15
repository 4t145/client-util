/// ok: `( U -> Result<T, E> ) -> ( U -> Option<T> )`
pub(crate) const fn ok<U, T, E>(
    map: impl FnOnce(U) -> Result<T, E>,
) -> impl FnOnce(U) -> Option<T> {
    move |u| map(u).ok()
}

/// a function wrapper for unreachable! macro
pub(crate) fn never<T, U>(_: T) -> U {
    unreachable!("I'll never let you down ~");
}

#[cfg(feature = "auth")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
pub fn basic_auth<U, P>(username: U, password: Option<P>) -> http::HeaderValue
where
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    use base64::prelude::BASE64_STANDARD;
    use base64::write::EncoderWriter;
    use http::HeaderValue;
    use std::io::Write;

    let mut buf = b"Basic ".to_vec();
    {
        let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
        let _ = write!(encoder, "{username}:");
        if let Some(password) = password {
            let _ = write!(encoder, "{password}");
        }
    }
    let mut header = HeaderValue::from_bytes(&buf).expect("base64 is always valid HeaderValue");
    header.set_sensitive(true);
    header
}

#[cfg(feature = "auth")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
pub fn bearer_auth<T>(token: T) -> http::HeaderValue
where
    T: std::fmt::Display,
{
    use http::HeaderValue;
    use std::io::Write;
    let mut buf = b"Bearer ".to_vec();
    let _ = write!(buf, "{token}");
    let mut header = HeaderValue::from_bytes(&buf).expect("token is always valid HeaderValue");
    header.set_sensitive(true);
    header
}

#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
pub(crate) fn fast_random() -> u64 {
    use std::cell::Cell;
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::num::Wrapping;

    thread_local! {
        static RNG: Cell<Wrapping<u64>> = Cell::new(Wrapping(seed()));
    }

    fn seed() -> u64 {
        let seed = RandomState::new();

        let mut out = 0;
        let mut cnt = 0;
        while out == 0 {
            cnt += 1;
            let mut hasher = seed.build_hasher();
            hasher.write_usize(cnt);
            out = hasher.finish();
        }
        out
    }

    RNG.with(|rng| {
        let mut n = rng.get();
        debug_assert_ne!(n.0, 0);
        n ^= n >> 12;
        n ^= n << 25;
        n ^= n >> 27;
        rng.set(n);
        n.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    })
}
