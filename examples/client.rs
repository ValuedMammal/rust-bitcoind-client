use bitcoind_client::http::{self, Auth};

/// RPC url.
const URL: &str = "http://127.0.0.1:18443";
/// Path to bitcoind cookie file.
const COOKIE_FILE: &str = ".bitcoin/regtest/.cookie";

fn main() -> anyhow::Result<()> {
    let path = std::env::var("RPC_COOKIE").unwrap_or(COOKIE_FILE.to_string());
    let client = http::Client::new(URL, Auth::CookieFile(path.into()))?;

    println!("{:#?}", client.get_blockchain_info()?);

    Ok(())
}
