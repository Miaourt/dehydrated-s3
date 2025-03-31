use std::str::FromStr;

use anyhow::{Context as _, Result, bail};
use http::uri::Scheme;
use smol::{
    Executor, future,
    io::{AsyncReadExt, AsyncWriteExt},
};

async fn client(ex: &Executor<'_>) -> Result<()> {
    let uri = http::Uri::from_str("https://httpbin.org/ip")?;
    let host = uri.host().context("cannot parse host")?;

    match uri.scheme_str() {
        Some("http") => {
            let port = uri.port_u16().unwrap_or(80);
            let mut sock = smol::net::TcpStream::connect((host, port)).await?;

            sock.write_all(
                "GET /ip HTTP/1.1\nHost: httpbin.org\nConnection: close\n\n\n".as_bytes(),
            )
            .await?;
            let mut resp = String::new();
            sock.read_to_string(&mut resp).await?;
            println!("{}", resp);
        }
        Some("https") => {
            let port = uri.port_u16().unwrap_or(443);
            let sock = smol::net::TcpStream::connect((host, port)).await?;
            let mut tls = async_native_tls::connect(host, sock).await?;

            tls.write_all(
                "GET /ip HTTP/1.1\nHost: httpbin.org\nConnection: close\n\n\n".as_bytes(),
            )
            .await?;
            let mut resp = String::new();
            tls.read_to_string(&mut resp).await?;
            println!("{}", resp);
        }
        scheme => bail!("unsupported scheme: {:?}", scheme),
    }

    Ok(())
}

pub fn main() -> Result<()> {
    let ex = Executor::new();

    future::block_on(ex.run(client(&ex)))
}
