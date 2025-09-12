use client_util::prelude::*;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
mod support;

#[tokio::test]
async fn text_part() {
    let form = Form::new().text("foo", "bar");

    let expected_body = format!(
        "\
         --{0}\r\n\
         Content-Disposition: form-data; name=\"foo\"\r\n\r\n\
         bar\r\n\
         --{0}--\r\n\
         ",
        form.boundary()
    );

    let ct = format!("multipart/form-data; boundary={}", form.boundary());

    let server = support::server::http(move |mut req| {
        let ct = ct.clone();
        let expected_body = expected_body.clone();
        async move {
            assert_eq!(req.method(), "POST");
            assert_eq!(req.headers()["content-type"], ct);
            assert_eq!(
                req.headers()["content-length"],
                expected_body.len().to_string()
            );

            let mut full: Vec<u8> = Vec::new();
            while let Some(item) = req.body_mut().frame().await {
                full.extend(&*item.unwrap().into_data().unwrap());
            }

            assert_eq!(full, expected_body.as_bytes());

            http::Response::default()
        }
    });

    let url = format!("http://{}/multipart/1", server.addr());
    let mut client = build_https_client().unwrap();
    let res = Request::post(&url)
        .multipart(form)
        .expect("fail to build request")
        .send(&mut client)
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

#[cfg(feature = "stream")]
#[tokio::test]
async fn stream_part() {
    use bytes::Bytes;
    use client_util::prelude::*;
    use futures_util::{future, stream};
    use http_body::Frame;
    let stream = stream(stream::once(future::ready(Ok::<
        _,
        client_util::error::BodyError,
    >(Frame::data(
        Bytes::from_static(b"part1 part2"),
    )))))
    .boxed_unsync();
    let part = Part::body(stream);

    let form = Form::new().text("foo", "bar").part("part_stream", part);

    let expected_body = format!(
        "\
         --{0}\r\n\
         Content-Disposition: form-data; name=\"foo\"\r\n\
         \r\n\
         bar\r\n\
         --{0}\r\n\
         Content-Disposition: form-data; name=\"part_stream\"\r\n\
         \r\n\
         part1 part2\r\n\
         --{0}--\r\n\
         ",
        form.boundary()
    );

    let ct = format!("multipart/form-data; boundary={}", form.boundary());

    let server = support::server::http(move |req| {
        let ct = ct.clone();
        let expected_body = expected_body.clone();
        async move {
            assert_eq!(req.method(), "POST");
            assert_eq!(req.headers()["content-type"], ct);
            assert_eq!(req.headers()["transfer-encoding"], "chunked");

            let full = req.collect().await.unwrap().to_bytes();

            assert_eq!(full, expected_body.as_bytes());

            http::Response::default()
        }
    });

    let url = format!("http://{}/multipart/1", server.addr());

    let res = Request::post(&url)
        .multipart(form)
        .expect("fail to build request")
        .send(build_https_client().unwrap())
        .await
        .expect("Failed to post multipart");
    assert_eq!(res.status(), http::StatusCode::OK);
}
