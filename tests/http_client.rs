//! Tests the RPC methods of the `http::Client`.

mod common;

use bitcoin::{Amount, BlockHash, Txid};
use bitcoind_client::jsonrpc::serde_json::json;
use bitcoind_client::types::ImportDescriptorsRequest;
use bitcoind_client::{Batch, Rpc};
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

#[test]
fn test_send_batch() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let best_hash = mined_block_hash(&env)?;
    let genesis_hash = env.client.get_block_hash(0)?;

    let mut batch = Batch::new(Rpc::GetBlockCount, vec![]);
    batch.push(Rpc::GetBestBlockHash, vec![]);
    batch.push(Rpc::GetBlockHash, vec![json!(0)]);

    let responses = env.client.send_batch(&batch)?;
    assert_eq!(responses.len(), 3, "expected 3 responses");

    let block_count: u32 = responses[0].result()?;
    assert_eq!(block_count, 1, "unexpected block count");

    let best_hash_res: BlockHash = responses[1].result::<String>()?.parse()?;
    assert_eq!(best_hash_res, best_hash, "unexpected best block hash");

    let genesis_hash_res: BlockHash = responses[2].result::<String>()?.parse()?;
    assert_eq!(genesis_hash_res, genesis_hash, "unexpected genesis block hash");

    Ok(())
}

#[test]
fn test_send_many_success() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let _ = env.mine_blocks(10, None)?;

    let heights = [1u32, 2, 3];
    let params = heights.iter().map(|h| vec![json!(h)]);

    use corepc_types::v31::GetBlockHash;
    let rpc = Rpc::GetBlockHash;
    let results = env.client.send_many::<GetBlockHash>(rpc, params)?;

    for (height, result) in heights.into_iter().zip(results) {
        let get_block_hash = env.bitcoind.client.get_block_hash(height as u64)?;
        assert_eq!(result.unwrap(), get_block_hash);
    }

    Ok(())
}

#[test]
fn test_send_many_result_error() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let _ = env.mine_blocks(10, None)?;

    let heights = [1u32, 2, 999]; // height 999 doesn't exist yet
    let params = heights.iter().map(|h| vec![json!(h)]);

    use corepc_types::v31::GetBlockHash;
    let rpc = Rpc::GetBlockHash;
    let results = env.client.send_many::<GetBlockHash>(rpc, params)?;

    for (height, result) in heights.into_iter().zip(results) {
        if height < 10 {
            result.expect("getblockhash should succeed");
        } else {
            result.expect_err("getblockhash should error for non-existing height");
        }
    }

    Ok(())
}

#[test]
fn test_send_accepts_method_name_string() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let address = env.bitcoind.client.new_address()?;

    let hashes: Vec<String> = env.client.send("generatetoaddress", &[json!(1), json!(address)])?;
    let hash: BlockHash = hashes[0].parse()?;

    assert_eq!(hash, env.client.get_best_block_hash()?, "unexpected mined block hash");
    Ok(())
}
