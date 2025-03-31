mod http_client;
mod hyper_client;
mod hyper_server;

use anyhow::Result;

fn main() -> Result<()> {
    // hyper_client::main()
    // hyper_server::server()
    http_client::main()
}
