pub const fn ok<U, T, E>(map: impl FnOnce(U) -> Result<T, E>) -> impl FnOnce(U) -> Option<T> {
    move |u| match map(u) {
        Ok(t) => Some(t),
        Err(_) => None,
    }
}

/// a function wrapper for unreachable! macro
pub fn never<T, U>(_: T) -> U {
    unreachable!("I'll never let you down ~");
}

#[cfg(feature = "auth")]
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
pub fn simple_rand() -> u64 {
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time is always after epoch")
        .as_nanos();
    use std::hash::{DefaultHasher, Hasher};
    let mut hasher = DefaultHasher::new();
    hasher.write_u128(timestamp);
    hasher.finish()
}
