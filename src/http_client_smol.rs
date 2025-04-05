// use std::io;
// use std::task::{Context, Poll};
// use std::{pin::Pin, str::FromStr};

// use anyhow::{Context as _, Result, bail};
// // use http::uri::Scheme;
// use async_native_tls::TlsStream;
// use smol::{
//     Executor, future,
//     io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
//     net::TcpStream,
// };

// async fn client(ex: &Executor<'_>) -> Result<()> {
//     let uri = http::Uri::from_str("https://httpbin.org/ip")?;
//     let host = uri.host().context("cannot parse host")?;

//     let mut sock = match uri.scheme_str() {
//         Some("http") => {
//             let port = uri.port_u16().unwrap_or(80);
//             let plain = smol::net::TcpStream::connect((host, port)).await?;
//             PlainOrTlsStream::Plain(plain)
//         }
//         Some("https") => {
//             let port = uri.port_u16().unwrap_or(443);
//             let sock = smol::net::TcpStream::connect((host, port)).await?;
//             let tls = async_native_tls::connect(host, sock).await?;
//             PlainOrTlsStream::Tls(tls)
//         }
//         scheme => bail!("unsupported scheme: {:?}", scheme),
//     };

//     sock.write_all("GET /ip HTTP/1.1\nHost: httpbin.org\nConnection: close\n\n\n".as_bytes())
//         .await?;
//     let mut resp = String::new();
//     sock.read_to_string(&mut resp).await?;
//     println!("{}", resp);

//     Ok(())
// }

// #[allow(dead_code)]
// pub fn main() -> Result<()> {
//     let ex = Executor::new();

//     future::block_on(ex.run(client(&ex)))
// }

// enum PlainOrTlsStream {
//     Plain(TcpStream),
//     Tls(TlsStream<TcpStream>),
// }
// impl AsyncRead for PlainOrTlsStream {
//     fn poll_read(
//         mut self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &mut [u8],
//     ) -> Poll<io::Result<usize>> {
//         match &mut *self {
//             PlainOrTlsStream::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
//             PlainOrTlsStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
//         }
//     }
// }

// impl AsyncWrite for PlainOrTlsStream {
//     fn poll_write(
//         mut self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &[u8],
//     ) -> Poll<io::Result<usize>> {
//         match &mut *self {
//             PlainOrTlsStream::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
//             PlainOrTlsStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
//         }
//     }

//     fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
//         match &mut *self {
//             PlainOrTlsStream::Plain(stream) => Pin::new(stream).poll_close(cx),
//             PlainOrTlsStream::Tls(stream) => Pin::new(stream).poll_close(cx),
//         }
//     }

//     fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
//         match &mut *self {
//             PlainOrTlsStream::Plain(stream) => Pin::new(stream).poll_flush(cx),
//             PlainOrTlsStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
//         }
//     }
// }
