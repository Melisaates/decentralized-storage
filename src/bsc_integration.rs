use ethers::{
    contract::Contract,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use std::sync::Arc;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BSCConfig {
    rpc_url: String,
    contract_address: String,
    private_key: String,
}

pub struct BSCIntegration {
    contract: Arc<Contract<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

impl BSCIntegration {
    pub async fn new(config: BSCConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect to BSC
        let provider = Provider::<Http>::try_from(config.rpc_url)?;
        let chain_id = provider.get_chainid().await?;

        // Setup wallet
        let wallet = LocalWallet::from_str(&config.private_key)?
            .with_chain_id(chain_id.as_u64());
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        // Contract ABI
        const ABI: &str = include_str!("./tuffle_project/build/contracts/StorageStaking.json");
        
        // Create contract instance
        let contract_addr = Address::from_str(&config.contract_address)?;
        let contract = Contract::new(contract_addr, serde_json::from_str(ABI)?, client);

        Ok(Self {
            contract: Arc::new(contract),
        })
    }

    pub async fn verify_stake(
        &self,
        user_address: &str,
    ) -> Result<(bool, u64), Box<dyn std::error::Error>> {
        let address = Address::from_str(user_address)?;
        
        // Call contract methods
        let is_active: bool = self
            .contract
            .method("isStakeActive", (address,))?
            .call()
            .await?;

        let storage_limit: U256 = self
            .contract
            .method("getStorageLimit", (address,))?
            .call()
            .await?;

        Ok((is_active, storage_limit.as_u64()))
    }

    pub async fn get_stake_info(
        &self,
        user_address: &str,
    ) -> Result<StakeInfo, Box<dyn std::error::Error>> {
        let address = Address::from_str(user_address)?;
        
        let (amount, timestamp, storage_limit, active): (U256, U256, U256, bool) = self
            .contract
            .method("getStakeInfo", (address,))?
            .call()
            .await?;

        Ok(StakeInfo {
            amount: amount.as_u64(),
            timestamp: timestamp.as_u64(),
            storage_limit: storage_limit.as_u64(),
            active,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakeInfo {
    pub amount: u64,
    pub timestamp: u64,
    pub storage_limit: u64,
    pub active: bool,
}

// Update PBE implementation to use BSC integration
impl ProgrammableBusinessEngine {
    pub async fn verify_smart_contract_stake(
        &self,
        bsc: &BSCIntegration,
        user_address: &str,
    ) -> Result<u64, String> {
        let (is_active, storage_limit) = bsc
            .verify_stake(user_address)
            .await
            .map_err(|e| format!("BSC verification failed: {}", e))?;

        if !is_active {
            return Err("No active stake found".to_string());
        }

        Ok(storage_limit)
    }

    pub async fn update_user_storage_limit(
        &mut self,
        bsc: &BSCIntegration,
        user_id: &str,
        user_address: &str,
    ) -> Result<(), String> {
        let stake_info = bsc
            .get_stake_info(user_address)
            .await
            .map_err(|e| format!("Failed to get stake info: {}", e))?;

        if !stake_info.active {
            return Err("No active stake found".to_string());
        }

        let token = StorageToken {
            user_id: user_id.to_string(),
            amount: stake_info.amount,
            storage_limit: stake_info.storage_limit,
            expiry: stake_info.timestamp + (30 * 24 * 60 * 60), // 30 days from stake timestamp
        };

        self.tokens.insert(user_id.to_string(), token);
        Ok(())
    }
}

// // Example usage
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let config = BSCConfig {
//         rpc_url: "https://data-seed-prebsc-1-s1.binance.org:8545".to_string(), // BSC Testnet
//         contract_address: "YOUR_CONTRACT_ADDRESS".to_string(),
//         private_key: "YOUR_PRIVATE_KEY".to_string(),
//     };

//     let bsc = BSCIntegration::new(config).await?;
//     let mut pbe = ProgrammableBusinessEngine::new(1_000_000);

//     // Verify user stake
//     let user_address = "USER_BSC_ADDRESS";
//     let storage_limit = pbe.verify_smart_contract_stake(&bsc, user_address).await?;
//     println!("User storage limit: {} bytes", storage_limit);

//     Ok(())
// }