use ethers::prelude::*;
use std::convert::TryFrom;

// Ethereum veya BSC ağına bağlanmak için RPC URL kullanabilirsiniz
const RPC_URL: &str = "https://bsc-dataseed.binance.org/";

pub async fn record_file_ownership(file_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(RPC_URL)?;
    let wallet: LocalWallet = "YOUR_PRIVATE_KEY".parse()?;
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    let contract_address = "0xYourContractAddress"; // Kontrat adresinizi buraya ekleyin
    let contract = Contract::from_json(
        client.clone(),
        contract_address.parse()?,
        include_bytes!("your_contract_abi.json"), // ABI dosyasını ekleyin
    )?;

    // Dosya hash'ini kontrata gönderelim
    let tx = contract
        .method::<_, H256>("recordFileOwnership", (file_hash.to_string(),))?
        .send()
        .await?;

    println!("Transaction hash: {:?}", tx);
    Ok(())
}
