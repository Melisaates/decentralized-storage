use ethers::prelude::*;
use std::sync::Arc;
use dotenv::dotenv;
use std::env;

pub struct BscClient {
    pub client: Arc<SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>>,
}

impl BscClient {
    /// Create a new Binance Smart Chain client
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let rpc_url = env::var("BSC_RPC_URL")?;
        let private_key = env::var("PRIVATE_KEY")?;

        let provider = Provider::<Http>::try_from(rpc_url)?;
        let wallet: LocalWallet = private_key.parse()?;
        let client = SignerMiddleware::new(provider, wallet);

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Send metadata
    pub async fn send_metadata(
        &self,
        contract_address: Address,
        file_id: &str,
        node_id: &str,
    ) -> Result<TxHash, Box<dyn std::error::Error>> {
        let abi = include_str!("../artifacts/contract_abi.json");
        let abi = serde_json::from_slice(abi.as_bytes())?;
        let contract = Contract::new(contract_address, abi, self.client.clone());

        let tx = contract
            .method::<(String, String), ()>("storeMetadata", (file_id.to_string(), node_id.to_string()))?
            .send()
            .await?;

        Ok(tx.tx_hash())
    }
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create BSC client
    let bsc_client = BscClient::new().await?;

    // Smart contract address
    let contract_address = "0x1234567890abcdef1234567890abcdef12345678".parse()?;

    // Send metadata
    let file_id = "example_file_id";
    let node_id = "example_node_id";
    let tx_hash = bsc_client
        .send_metadata(contract_address, file_id, node_id)
        .await?;

    println!("Transaction hash: {}", tx_hash);

    Ok(())
}
 */
