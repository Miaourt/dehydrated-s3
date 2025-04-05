use std::str::FromStr;

use anyhow::{Context as _, Result, bail};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio_native_tls::native_tls::TlsConnector;

async fn client(mut sock: impl AsyncRead + AsyncWrite + Unpin) -> Result<()> {
    sock.write_all("GET /ip HTTP/1.1\nHost: httpbin.org\nConnection: close\n\n\n".as_bytes())
        .await?;
    let mut resp = String::new();
    sock.read_to_string(&mut resp).await?;
    println!("{}", resp);

    Ok(())
}

async fn connect() -> Result<()> {
    let uri = http::Uri::from_str("https://httpbin.org/ip")?;
    let host = uri.host().context("cannot parse host")?;

    match uri.scheme_str() {
        Some("http") => {
            let port = uri.port_u16().unwrap_or(80);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            client(plain).await
        }
        Some("https") => {
            let port = uri.port_u16().unwrap_or(443);
            let plain = tokio::net::TcpStream::connect((host, port)).await?;
            let cx = TlsConnector::builder().build()?;
            let cx = tokio_native_tls::TlsConnector::from(cx);
            let tls = cx.connect(host, plain).await?;
            client(tls).await
        }
        scheme => bail!("unsupported scheme: {:?}", scheme),
    }
}

pub fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(connect())
}
