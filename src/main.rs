mod constants;
mod database;
mod file_scanner;
mod procedures;

use anyhow::Result;
use jsonrpsee::server::ServerBuilder;

#[tokio::main]
async fn main() -> Result<()> {
  let server = ServerBuilder::default().build("0.0.0.0:25194").await?;
  let handle = server.start(procedures::build_rpc_module()?);

  println!("Glean JSON-RPC server listening on 0.0.0.0:25194");

  handle.stopped().await;
  Ok(())
}
