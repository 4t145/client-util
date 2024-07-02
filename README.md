# client-util
[![Crates.io Version](https://img.shields.io/crates/v/client-util)](https://crates.io/crates/client-util)
![Release status](https://github.com/4t145/client-util/actions/workflows/test-and-release.yml/badge.svg)
[![docs.rs](https://img.shields.io/docsrs/client-util)](https://docs.rs/client-util)

Help you to build requests and handle responses by several extension trait!

## Usage
```bash
cargo add client-util
```

### Make it easier to use hyper http client
```rust
use client_util::prelude::{RequestBuilderExt, RequestExt, ResponseExt, hyper_tls_client};
#[tokio::main]
async fn main() -> client_util::Result<()> {
    let mut client = hyper_tls_client();

    let request = http::Request::get("https://httpbin.org/json")
        .version(http::Version::HTTP_11)
        .json("hello client-util")?;

    let (parts, response) = request
        .send(&mut client)
        .await?
        .json::<serde_json::Value>()
        .await?
        .into_parts();
    println!("{:?}", parts);
    println!("{:?}", response);

    Ok(())
}
```

### Customize your own client

In [`RequestExt`](`crate::request::RequestExt`) trait, we send the request by a tower service, so you can add any middle layer on the top of the existed client.

## What about...
Theoretically, you can add any feature to any client by using [`tower`](https://docs.rs/tower).

###  What about trace, metrics, following redirect and more features?

You can find those features in [`tower-http`](./https://docs.rs/tower-http/latest/tower_http/) crate as tower layers.

###  What about cookies?

We have [`tower-cookie`](https://crates.io/crates/tower-cookies)

## Feature Flags
|flag                           |description                                |
|:------------------------------|:------------------------------------------|
|json                           |json body                                  |
|form                           |form body                                  |
|multipart                      |multipart form body                        |
|query                          |serialize into and append url's query      |
|auth                           |method to append auth header               |
|decompression-deflate          |deflate decompression, need tokio runtime  |
|decompression-gzip             |gzip decompression, need tokio runtime     |
|decompression-br               |br decompression, need tokio runtime       |
|decompression-zstd             |zstd decompression, need tokio runtime     |
|decompression-all              |all decompression support upon             |
|hyper-client                   |shortcut to create a hyper http client     |
|hyper-client-rustls            |hyper-client with rustls                   |
|rt-tokio                       |run with tokio runtime, which allows you use [`tower-http`](./https://docs.rs/tower-http/latest/tower_http/) |