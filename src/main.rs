mod constant;
mod database;
mod file_scanner;
mod procedure;

use anyhow::{Result, anyhow};
use jsonrpsee::server::ServerBuilder;

use database::DatabaseInitializer;

fn main() -> Result<()> {
  DatabaseInitializer::initialize().map_err(|err| anyhow!(err.to_string()))?;

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();

  rt.block_on(async {
    let server = ServerBuilder::default().build("127.0.0.1:25194").await?;
    let handle = server.start(procedure::build_rpc_module()?);
    println!("Glean JSON-RPC server listening on 127.0.0.1:25194");
    handle.stopped().await;
    Ok::<(), anyhow::Error>(())
  })?;

  Ok(())
}
