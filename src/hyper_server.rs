use std::{net::TcpListener, sync::Arc, time::Duration};

use anyhow::{Error, Result};
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use smol::{Async, Executor, Timer, future};
use smol_hyper::{rt::FuturesIo, rt::SmolTimer};

/// Serves a request and returns a response.
async fn serve(req: Request<Incoming>) -> Result<Response<Full<&'static [u8]>>> {
    println!("Serving {}", req.uri());
    Ok(Response::new(Full::new("Hello from hyper!".as_bytes())))
}
/// Listens for incoming connections and serves them.
async fn listen(ex: Arc<Executor<'_>>, listener: Async<TcpListener>) -> Result<()> {
    println!("Listening on http://{}", listener.get_ref().local_addr()?);

    loop {
        // Accept the next connection.
        let (stream, _) = listener.accept().await?;

        // Spawn a background task serving this connection.
        let task = ex.spawn(async move {
            println!("Quoicou");

            if let Err(err) = http1::Builder::new()
                .timer(SmolTimer::new())
                .serve_connection(FuturesIo::new(stream), service_fn(serve))
                .await
            {
                println!("Connection error: {:#?}", err);
            }

            println!("Feur");
        });

        // Detach the task to let it run in the executor.
        task.detach();
    }
}

#[allow(dead_code)]
pub fn server() -> Result<()> {
    // thanks to the termination feature of ctrlc,
    // also handle SIGINT, SIGTERM and SIGHUP.
    let ctrl_c = {
        let (s, ctrl_c) = smol::channel::bounded(100);
        ctrlc::set_handler(move || {
            s.try_send(()).unwrap();
        })
        .expect("error while creating the ctrl_c handler");
        ctrl_c
    };

    let ex = Arc::new(Executor::new());

    future::block_on(ex.run(async {
        let http = listen(
            ex.clone(),
            Async::<TcpListener>::bind(([127, 0, 0, 1], 8080))?,
        );
        let termination = async { ctrl_c.recv().await.map_err(|e| Error::new(e)) };

        if let Err(err) = future::or(http, termination).await {
            println!("Server error: {:#?}", err);
        }

        println!("Shutting down the server...");
        println!("Waiting for remaining requests...");

        loop {
            if ex.is_empty() {
                println!("No more requests, goodbye !");
                return Ok(());
            }
            Timer::after(Duration::from_millis(100)).await;
        }
    }))
}
