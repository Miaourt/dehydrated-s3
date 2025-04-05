mod http_client_smol;
mod http_client_tokio;
mod hyper_client;
mod hyper_server;

use anyhow::Result;

fn main() -> Result<()> {
    // hyper_client::main()
    // hyper_server::server()
    // http_client_smol::main()
    http_client_tokio::main()
}
