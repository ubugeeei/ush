mod convert;
mod document;
mod server;

use anyhow::Result;
use lsp_server::Connection;

fn main() -> Result<()> {
    let (connection, io_threads) = Connection::stdio();
    server::run(connection)?;
    io_threads.join()?;
    Ok(())
}
