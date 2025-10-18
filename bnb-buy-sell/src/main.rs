use ethers::prelude::*;
use std::{env, io, str::FromStr, sync::Arc, time::Duration};
use dotenv::dotenv;
use anyhow::{Context, Result};

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
        function approve(address spender, uint256 amount) returns (bool)
    ]"#
);

abigen!(
    TokenManagerHelper3,
    r#"[
        function tryBuy(address token, uint256 amount, uint256 funds) view returns (address tokenManager, address quote, uint256 estimatedAmount, uint256 estimatedCost, uint256 estimatedFee, uint256 amountMsgValue, uint256 amountApproval, uint256 amountFunds)
        function trySell(address token, uint256 amount) view returns (address tokenManager, address quote, uint256 funds, uint256 fee)
   ]"#
);

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // 🔐 Load environment variables
    let private_key = env::var("PRIVATE_KEY").context("Missing PRIVATE_KEY in .env")?;
    let rpc_url = env::var("RPC_URL").context("Missing RPC_URL in .env")?;
    let manager_address: Address = env::var("TOKEN_MANAGER2")?.parse()?;
    let token_address: Address = env::var("TOKEN_ADDRESS")?.parse()?;
    let helper3_address: Address = env::var("HELPER3_ADDRESS")?.parse()?;

    // 🌐 Initialize provider + wallet
    let provider = Provider::<Http>::try_from(rpc_url)?
        .interval(Duration::from_millis(6000));

    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(56u64);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // 🧠 Initialize contracts
    let token_manager = TokenManager2::new(manager_address, client.clone());
    let helper = TokenManagerHelper3::new(helper3_address, client.clone());
    let token = ERC20::new(token_address, client.clone());

    // 🧭 Choose function
    println!("\nChoose an action:");
    println!("1) buyToken (buy exact amount)");
    println!("2) sellToken (sell tokens)");
    println!("3) buyTokenAMAP (spend fixed BNB)");
    print!("> ");
    use std::io::Write;
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        // ✅ BUY EXACT TOKEN AMOUNT
        "1" => {
            println!("🟢 Running buyToken...");
            let call = token.approve(manager_address, U256::MAX);
            let approve_call = token.approve(manager_address, U256::MAX);
            let approve_tx = approve_call
                .send()
                .await
                .context("Failed to approve TokenManager2")?;
            println!("✅ Approval tx: {:?}", approve_tx.tx_hash());

            println!("Enter token amount (whole tokens): ");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let token_amount: f64 = input.trim().parse().context("Invalid number")?;
            let buy_amount = U256::from((token_amount * 1e18_f64) as u128);

            ///
            println!("✅ buyToken tx sent: {:?}", tx.tx_hash());
        }

        // ✅ SELL EXACT TOKEN AMOUNT
        "2" => {
            println!("🔵 Running sellToken...");
            println!("Enter token amount to sell (whole tokens): ");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let token_amount: f64 = input.trim().parse().context("Invalid number")?;
            let amount_to_sell = U256::from((token_amount * 1e18_f64) as u128);

            /////
        }

        // ✅ BUY WITH FIXED BNB
        "3" => {
            println!("🟣 Running buyTokenAMAP...");
            println!("Enter BNB amount to spend (e.g. 0.05): ");
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let bnb_amount: f64 = input.trim().parse().context("Invalid number")?;
            let funds_to_spend = U256::from((bnb_amount * 1e18_f64) as u128);

            println!("Estimating tokens you can buy...");
            let (_, _, estimated_tokens, _, _, _, _, _) =
                helper.try_buy(token_address, U256::zero(), funds_to_spend).call().await?;

            println!(
                "💰 You’ll likely receive ~{} tokens for {} BNB",
                estimated_tokens / U256::exp10(18),
                bnb_amount
            );

            println!("Proceed with transaction? (y/n): ");
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;
            if confirm.trim().to_lowercase() != "y" {
                println!("❌ Transaction cancelled.");
                return Ok(());
            }

            ////
        }

        
        _ => println!("❌ Invalid option. Choose 1, 2 or 3."),
    }

    Ok(())
}
