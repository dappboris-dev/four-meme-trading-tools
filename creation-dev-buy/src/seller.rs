use std::sync::Arc;
use tokio::sync::mpsc;
use crate::types::BotEvent;
use ethers::prelude::*;
use dotenv::dotenv;
use std::time::Duration;
use anyhow::{Context, Result};
use std::env;

abigen!(
    TokenManager2,
    r#"[ 
        function buyToken(address token, uint256 amount, uint256 maxFunds) payable
        function buyTokenAMAP(address token, uint256 funds, uint256 minAmount) payable
        function sellToken(address token, uint256 amount)
        function sellTokenAMAP(address token, uint256 amount, uint256 minBNB)
    ]"#
);

abigen!(
    ERC20,
    r#"[ 
        function balanceOf(address) view returns (uint256)
        function approve(address spender, uint256 amount) returns (bool)
        function decimals() view returns (uint8)
    ]"#
);

#[derive(Debug)]
pub struct TokenBalance {
    pub raw: U256,
    pub formatted: f64,
    pub wallet: Address,
    pub token: Address,
}

pub struct Seller {
    receiver: mpsc::Receiver<BotEvent>,
}

impl Seller {
    pub fn new(receiver: mpsc::Receiver<BotEvent>) -> Self {
        Self { receiver }
    }

    // Async helper to get token balance and decimals
    pub async fn get_wallet_token_balance(
        client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
        token_address: Address,
    ) -> Result<(U256, u8)> {
        let token = ERC20::new(token_address, client.clone());
        let wallet_address = client.address();

        let balance = token.balance_of(wallet_address)
            .call()
            .await
            .context("Failed to get token balance")?;
        let decimals = token.decimals()
            .call()
            .await
            .context("Failed to get token decimals")?;

        Ok((balance, decimals))
    }

    pub async fn run(&mut self) -> Result<()> {
        dotenv().ok();

        let private_key = env::var("PRIVATE_KEY").context("Missing PRIVATE_KEY")?;
        let rpc_url = env::var("RPC_URL").context("Missing RPC_URL")?;
        let manager_address: Address = env::var("TOKEN_MANAGER_ADDRESS")?.parse()?;

        let provider = Provider::<Http>::try_from(rpc_url)?.interval(Duration::from_millis(300));
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(56u64);
        let client = Arc::new(SignerMiddleware::new(provider, wallet));
        let token_manager = Arc::new(TokenManager2::new(manager_address, client.clone()));

        println!("🟢 Seller started and listening for SellRequest events...");

        while let Some(event) = self.receiver.recv().await {
            if let BotEvent::SellRequest { token_address } = event {
                println!("📨 Received SellRequest for token: {:?}", token_address);

                let client = client.clone();
                let token_manager = token_manager.clone();
                let token_address_copy = token_address;
           

        Ok(())
    }
}
