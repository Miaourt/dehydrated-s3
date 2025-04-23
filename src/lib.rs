use std::str::FromStr;

use anyhow::{Context as _, Result, bail};

use http::Response;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_native_tls::native_tls::TlsConnector;

mod awsv4;

fn build_response(
    resp: httparse::Response,
    body_buf: Vec<u8>,
    body_start: u64,
    sock: impl AsyncRead + AsyncWrite + Unpin + 'static,
) -> http::Response<Box<dyn AsyncRead + Unpin>> {
    let mut builder = Response::builder();

    if let Some(code) = resp.code {
        builder = builder.status(code);
    }

    if let Some(version) = resp.version {
        builder = builder.version(match version {
            1 => http::Version::HTTP_11,
            0 => http::Version::HTTP_10,
            _ => http::Version::HTTP_11,
        });
    }

    for header in resp.headers.iter() {
        builder = builder.header(header.name, header.value);
    }

    println!("{:?}", String::from_utf8(body_buf.clone()).unwrap());
    let mut buf = std::io::Cursor::new(body_buf);
    buf.set_position(0);

    let mut res = Vec::new();
    println!("{:?}", String::from_utf8(res).unwrap());

    let body: Box<dyn AsyncRead + Unpin> = Box::new(buf);

    let resp = builder.body(body).unwrap();

    return resp;
}

async fn parse_header(
    mut sock: impl AsyncRead + AsyncWrite + Unpin + 'static,
    req: &[u8],
) -> Result<http::Response<Box<dyn AsyncRead + Unpin>>> {
    sock.write_all(req).await?;

    let mut buf = vec![0u8; 8192];
    let mut total_bytes_read = 0;

    loop {
        let bytes_read = sock.read(&mut buf[total_bytes_read..]).await?;
        if bytes_read == 0 {
            bail!("Connection closed before receiving complete HTTP response")
        }

        total_bytes_read += bytes_read;

        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut r = httparse::Response::new(&mut headers);

        match r.parse(&buf[..total_bytes_read]) {
            Ok(status) => {
                if status.is_complete() {
                    let body_start = status.unwrap() as u64;
                    return Ok(build_response(r, buf.to_vec(), body_start, sock));
                }
            }
            Err(e) => {
                bail!("Error while parsing the http request: {}", e);
            }
        }

        if total_bytes_read == buf.len() {
            bail!("Response too large for buffer");
        }
    }
}

pub async fn do_http(
    endpoint: &str,
    req: &str,
) -> Result<http::Response<Box<dyn AsyncRead + Unpin>>> {
    let uri = http::Uri::from_str(endpoint)?;
    let host = uri.host().context("cannot parse host")?;

    match uri.scheme_str() {
        Some("http") => {
            let port = uri.port_u16().unwrap_or(80);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            parse_header(plain, req.as_bytes()).await
        }
        Some("https") => {
            let port = uri.port_u16().unwrap_or(443);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            let cx = TlsConnector::builder().build()?;
            let cx = tokio_native_tls::TlsConnector::from(cx);
            let tls = cx.connect(host, plain).await?;
            parse_header(tls, req.as_bytes()).await
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

    let resp = do_http("https://httpbin.org", req).await;
    assert!(resp.is_ok());

    let mut resp = resp.unwrap();
    // println!("{:?}", resp.headers());

    let mut buf = vec![0u8; 8192];

    resp.body_mut().read_to_end(&mut buf);
    // println!("{:?}", String::from_utf8(buf).unwrap());
}
