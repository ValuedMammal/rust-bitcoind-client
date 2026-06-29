//! Tests the RPC methods of the `simple_http::Client`.

mod common;

use bitcoin::{Amount, BlockHash, Txid};
use bitcoind_client::types::ImportDescriptorsRequest;
use common::TestEnv;
use corepc_types::bitcoin;

fn mined_block_hash(env: &TestEnv) -> anyhow::Result<BlockHash> {
    Ok(env.mine_blocks(1, None)?[0])
}

fn funded_send(env: &TestEnv) -> anyhow::Result<Txid> {
    env.mine_blocks(101, None)?;
    let address = env.bitcoind.client.new_address()?;
    Ok(env.client.send_to_address(&address, Amount::from_sat(50_000))?)
}

fn print_subversion_string(env: &TestEnv) -> anyhow::Result<()> {
    println!("{}", env.bitcoind.client.get_network_info()?.subversion);
    Ok(())
}

#[test]
fn test_get_blockchain_info() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    print_subversion_string(&env)?;
    let result = env.client.get_blockchain_info();
    assert!(result.is_ok(), "failed to call getblockchaininfo: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_count() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let result = env.client.get_block_count();
    assert!(result.is_ok(), "failed to call getblockcount: {result:?}");
    Ok(())
}

#[test]
fn test_get_best_block_hash() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let result = env.client.get_best_block_hash();
    assert!(result.is_ok(), "failed to call getbestblockhash: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_hash() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let result = env.client.get_block_hash(0);
    assert!(result.is_ok(), "failed to call getblockhash: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_filter() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_filter(&hash);
    assert!(result.is_ok(), "failed to call getblockfilter: {result:?}");
    Ok(())
}

#[test]
fn test_get_block_raw() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_raw(&hash);
    assert!(result.is_ok(), "failed to call getblock raw: {result:?}");
    Ok(())
}

#[test]
fn test_get_block() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block(&hash);
    assert!(result.is_ok(), "failed to call getblock: {result:?}");
    Ok(())
}

#[test]
fn test_get_raw_mempool() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    env.mine_blocks(101, None)?;
    // Send tx to mempool
    let address = env.bitcoind.client.new_address()?;
    let txid = env
        .client
        .send_to_address(&address, Amount::from_sat(50_000))
        .expect("failed to send_to_address");

    // Get raw mempool txids
    let txids = env.client.get_raw_mempool().expect("failed get_raw_mempool");
    assert!(txids.contains(&txid), "unexpected mempool txid");

    // Get raw mempool (verbose)
    let mempool_entries = env
        .client
        .get_raw_mempool_verbose()
        .expect("failed get_raw_mempool_verbose");
    assert!(!mempool_entries.is_empty(), "tx should appear in mempool");
    let entry_txid = mempool_entries.keys().next().copied().unwrap();
    assert_eq!(entry_txid, txid, "unexpected mempool entry txid");
    Ok(())
}

#[test]
fn test_send_to_address() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    env.mine_blocks(101, None)?;
    let address = env.bitcoind.client.new_address()?;
    let result = env.client.send_to_address(&address, Amount::from_sat(50_000));
    assert!(result.is_ok(), "failed to call sendtoaddress: {result:?}");
    Ok(())
}

#[test]
fn test_get_raw_transaction() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let txid = funded_send(&env)?;
    let result = env.client.get_raw_transaction(&txid);
    assert!(result.is_ok(), "failed to call getrawtransaction: {result:?}");
    Ok(())
}

#[test]
fn test_import_descriptors() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let address = env.bitcoind.client.new_address()?;
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

#[test]
fn test_get_block_header() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let _header = env.client.get_block_header(&hash).expect("failed get_block_header");
    let _get_block_header_verbose = env
        .client
        .get_block_header_verbose(&hash)
        .expect("failed get_block_header_verbose");
    Ok(())
}

#[test]
fn test_get_block_verbose() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let hash = mined_block_hash(&env)?;
    let result = env.client.get_block_verbose(&hash);
    assert!(result.is_ok(), "failed to call getblock verbose: {result:?}");
    Ok(())
}

#[test]
fn test_get_descriptor_info() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let address = env.bitcoind.client.new_address()?;
    let descriptor = format!("addr({address})");
    let result = env.client.get_descriptor_info(&descriptor);
    assert!(result.is_ok(), "failed to call getdescriptorinfo: {result:?}");
    Ok(())
}
