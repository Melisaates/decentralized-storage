use ethers::prelude::*;
use std::sync::Arc;
use dotenv::dotenv;
use std::env;

pub struct BscClient {
    pub client: Arc<SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>>,
}

impl BscClient {
    /// Yeni bir Binance Smart Chain istemcisi oluştur
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

    /// Metadata gönderimi
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
