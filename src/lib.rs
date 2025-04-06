use std::str::FromStr;

use anyhow::{Context as _, Result, bail};

use http::Response;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_native_tls::native_tls::TlsConnector;

// The impl thingy is to leverage static dispatch and not doing `dyn Trait` yay
async fn client(
    mut sock: impl AsyncRead + AsyncWrite + Unpin,
    req: &[u8],
) -> Result<http::Response<()>> {
    sock.write_all(req).await?;

    let mut buf = vec![0u8; 8192];
    let mut total_bytes_read = 0;

    loop {
        let bytes_read = sock.read(&mut buf[total_bytes_read..]).await?;
        if bytes_read == 0 {
            break;
        }

        total_bytes_read += bytes_read;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut resp = httparse::Response::new(&mut headers);

        match resp.parse(&buf[..total_bytes_read]) {
            Ok(status) => {
                if status.is_complete() {
                    println!("{:?}", resp);
                    return Ok(Response::new(()));
                }
            }
            Err(e) => {
                if total_bytes_read == buf.len() {
                    bail!("Response too large for buffer: {}", e);
                }
            }
        }

        if total_bytes_read == buf.len() {
            bail!("Response too large for buffer");
        }
    }

    bail!("Connection closed before receiving complete HTTP response")
}

pub async fn do_http(endpoint: &str, req: &str) -> Result<http::Response<()>> {
    let uri = http::Uri::from_str(endpoint)?;
    let host = uri.host().context("cannot parse host")?;

    match uri.scheme_str() {
        Some("http") => {
            let port = uri.port_u16().unwrap_or(80);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            client(plain, req.as_bytes()).await
        }
        Some("https") => {
            let port = uri.port_u16().unwrap_or(443);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            let cx = TlsConnector::builder().build()?;
            let cx = tokio_native_tls::TlsConnector::from(cx);
            let tls = cx.connect(host, plain).await?;
            client(tls, req.as_bytes()).await
        }
        scheme => bail!("unsupported scheme: {:?}", scheme),
    }
}

#[tokio::test]
async fn my_test() {
    let req = "\
    GET /ip HTTP/1.1\n\
    Host: httpbin.org\n\
    Connection: close\n\
    \n";

    assert!(do_http("https://httpbin.org", req).await.is_ok());
}
