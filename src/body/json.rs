pub struct Json<T>(pub T);

impl<T> Json<T> {
    pub fn new(inner: T) -> Self {
        Json(inner)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use hyper::body::{Body, Bytes, Frame, SizeHint};
use serde::Serialize;
impl<T> Body for Json<T>
where
    T: Serialize,
{
    type Data = Bytes;
    type Error = serde_json::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut buf = Vec::new();
        serde_json::to_writer(&mut buf, &self.0).map_err(serde_json::Error::from)?;
        Poll::Ready(Some(Ok(Frame::data(Bytes::from(buf)))))
    }

    fn is_end_stream(&self) -> bool {
        true
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(1)
    }
}
