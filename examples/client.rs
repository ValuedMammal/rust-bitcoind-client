use simplerpc::simple_http::{self, Auth};

/// RPC url.
const URL: &str = "http://127.0.0.1:38332";
/// Path to bitcoind cookie file.
const COOKIE_FILE: &str = ".bitcoin/signet/.cookie";

fn main() -> anyhow::Result<()> {
    let cookie_file = std::env::var("RPC_COOKIE").unwrap_or(COOKIE_FILE.to_string());
    let client = simple_http::Client::new(URL, Auth::CookieFile(cookie_file.into()))?;

    println!("{:#?}", client.get_blockchain_info()?);

    Ok(())
}
