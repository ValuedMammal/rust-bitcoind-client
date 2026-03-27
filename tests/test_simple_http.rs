//! Tests the RPC methods of the `simple_http::Client`.

mod common;

use bitcoin::{Amount, BlockHash, Txid};
use bitcoind_client::types::ImportDescriptorsRequest;
use corepc_types::bitcoin;

fn mined_block_hash(env: &common::TestEnv) -> anyhow::Result<BlockHash> {
    Ok(env.mine_blocks(1, None)?[0])
}

fn funded_send(env: &common::TestEnv) -> anyhow::Result<Txid> {
    env.mine_blocks(101, None)?;
    let address = env.node.client.new_address()?;
    Ok(env.client.send_to_address(&address, Amount::from_sat(50_000))?)
}

#[test]
fn test_get_blockchain_info() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_blockchain_info();
    assert!(result.is_ok(), "failed to call getblockchaininfo: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_count() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_block_count();
    assert!(result.is_ok(), "failed to call getblockcount: {result:?}");
    Ok(())
}

#[test]
fn test_get_best_block_hash() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_best_block_hash();
    assert!(result.is_ok(), "failed to call getbestblockhash: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_hash() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_block_hash(0);
    assert!(result.is_ok(), "failed to call getblockhash: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_filter() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_filter(&hash);
    assert!(result.is_ok(), "failed to call getblockfilter: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_raw() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_raw(&hash);
    assert!(result.is_ok(), "failed to call getblock raw: {result:?}");
    Ok(())
}

#[test]
fn test_get_block() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block(&hash);
    assert!(result.is_ok(), "failed to call getblock: {result:?}");
    Ok(())
}

#[test]
fn test_get_rawm_empool() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_raw_mempool();
    assert!(result.is_ok(), "failed to call getrawmempool: {result:?}");
    Ok(())
}

#[test]
fn test_get_raw_mempool_verbose() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let result = env.client.get_raw_mempool_verbose();
    assert!(result.is_ok(), "failed to call getrawmempool verbose: {result:?}");
    Ok(())
}

#[test]
fn test_send_to_address() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    env.mine_blocks(101, None)?;
    let address = env.node.client.new_address()?;
    let result = env.client.send_to_address(&address, Amount::from_sat(50_000));
    assert!(result.is_ok(), "failed to call sendtoaddress: {result:?}");
    Ok(())
}

#[test]
fn test_get_raw_transaction() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let txid = funded_send(&env)?;
    let result = env.client.get_raw_transaction(&txid);
    assert!(result.is_ok(), "failed to call getrawtransaction: {result:?}");
    Ok(())
}

#[test]
fn test_import_descriptors() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let address = env.node.client.new_address()?;
    let request = ImportDescriptorsRequest {
        desc: format!("addr({address})"),
        timestamp: 0,
        ..Default::default()
    };
    let result = env.client.import_descriptors(&[request]);
    assert!(result.is_ok(), "failed to call importdescriptors: {result:?}");
    Ok(())
}

#[test]
#[ignore = "unimplemented"]
fn test_estimatesmartfee() -> anyhow::Result<()> {
    todo!()
}

#[cfg(not(feature = "28_0"))]
#[test]
fn test_get_block_header() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_header_verbose(&hash);
    assert!(result.is_ok(), "failed to call getblockheader: {result:?}");
    Ok(())
}

#[cfg(not(feature = "28_0"))]
#[test]
fn test_get_block_verbose() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_verbose(&hash);
    assert!(result.is_ok(), "failed to call getblock verbose: {result:?}");
    Ok(())
}

#[cfg(not(feature = "28_0"))]
#[test]
fn test_get_descriptor_info() -> anyhow::Result<()> {
    let env = common::TestEnv::new()?;
    let address = env.node.client.new_address()?;
    let descriptor = format!("addr({address})");
    let result = env.client.get_descriptor_info(&descriptor);
    assert!(result.is_ok(), "failed to call getdescriptorinfo: {result:?}");
    Ok(())
}
