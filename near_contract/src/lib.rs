use near_sdk::{
    env, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, UnorderedMap, Vector, log, Promise, ext_contract,
};
use near_sdk::serde::{Serialize, Deserialize};

// External contract interface for Aurora (placeholder for cross-chain calls)
#[ext_contract(ext_aurora)]
trait AuroraTrader {
    fn execute_trade(&self, user: AccountId, amount: Balance, asset: String);
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PortfolioData {
    assets: Vec<Asset>,         // List of user-held assets
    risk_tolerance: u8,         // 1-10 scale (low to high risk)
    last_updated: u64,          // Timestamp (nanoseconds)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Asset {
    name: String,               // e.g., "NEAR", "ETH"
    amount: Balance,            // Amount in smallest unit (e.g., yoctoNEAR)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TradeRequest {
    asset: String,
    amount: Balance,
    action: String,             // "buy" or "sell"
    timestamp: u64,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PortfolioManager {
    portfolios: UnorderedMap<AccountId, PortfolioData>,      // User portfolios
    ai_recommendations: UnorderedMap<AccountId, String>,     // AI suggestions (JSON)
    trade_history: UnorderedMap<AccountId, Vector<TradeRequest>>, // Trade requests per user
}

#[near_bindgen]
impl PortfolioManager {
    #[init]
    pub fn new() -> Self {
        Self {
            portfolios: UnorderedMap::new(b"p"),
            ai_recommendations: UnorderedMap::new(b"a"),
            trade_history: UnorderedMap::new(b"t"),
        }
    }

    /// Update user's portfolio with encrypted or raw data
    pub fn update_portfolio(&mut self, assets: Vec<Asset>, risk_tolerance: u8) {
        let account_id = env::predecessor_account_id();
        let portfolio = PortfolioData {
            assets,
            risk_tolerance: risk_tolerance.clamp(1, 10),
            last_updated: env::block_timestamp(),
        };
        self.portfolios.insert(&account_id, &portfolio);
        log!("Portfolio updated for {}", account_id);
    }

    /// Retrieve user's portfolio
    pub fn get_portfolio(&self) -> Option<PortfolioData> {
        let account_id = env::predecessor_account_id();
        self.portfolios.get(&account_id)
    }

    /// Store AI-generated recommendation (e.g., from an off-chain agent)
    pub fn set_ai_recommendation(&mut self, recommendation: String) {
        let account_id = env::predecessor_account_id();
        self.ai_recommendations.insert(&account_id, &recommendation);
        log!("AI recommendation set for {}", account_id);
    }

    /// Get AI recommendation
    pub fn get_ai_recommendation(&self) -> Option<String> {
        let account_id = env::predecessor_account_id();
        self.ai_recommendations.get(&account_id)
    }

    /// Request a trade to be executed on Aurora
    #[payable]
    pub fn request_trade(&mut self, asset: String, amount: Balance, action: String) {
        let account_id = env::predecessor_account_id();
        let deposit: Balance = env::attached_deposit(); // Require some NEAR for gas
        assert!(deposit > 0, "Attach NEAR for gas fees");
        assert!(action == "buy" || action == "sell", "Invalid action");

        // Record trade request
        let trade = TradeRequest {
            asset: asset.clone(),
            amount,
            action: action.clone(),
            timestamp: env::block_timestamp(),
        };
        let mut user_trades = self.trade_history.get(&account_id).unwrap_or_else(|| {
            Vector::new(format!("t-{}", account_id).as_bytes().to_vec())
        });
        user_trades.push(&trade);
        self.trade_history.insert(&account_id, &user_trades);

        // Emit event for Aurora (simulated promise, replace with real bridge call if available)
        Promise::new("aurora-trader.testnet".parse().unwrap()) // Replace with actual Aurora contract
            .function_call(
                "execute_trade".to_string(),
                near_sdk::serde_json::to_vec(&near_sdk::serde_json::json!({
                    "user": account_id,
                    "amount": amount,
                    "asset": asset
                }))
                .unwrap(),
                deposit,
                Gas::from(30_000_000_000_000), // 30 TGas
            );

        log!("Trade requested: {} {} {} by {}", action, amount, asset, account_id);
    }

    /// Get user's trade history
    pub fn get_trade_history(&self) -> Vec<TradeRequest> {
        let account_id = env::predecessor_account_id();
        self.trade_history
            .get(&account_id)
            .map(|history| history.to_vec())
            .unwrap_or_else(Vec::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    fn setup() -> (PortfolioManager, VMContextBuilder) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        (PortfolioManager::new(), context)
    }

    #[test]
    fn test_portfolio_management() {
        let (mut contract, mut context) = setup();
        let assets = vec![Asset { name: "NEAR".to_string(), amount: 10_000_000_000_000 }];
        contract.update_portfolio(assets.clone(), 5);
        let portfolio = contract.get_portfolio().unwrap();
        assert_eq!(portfolio.assets[0].name, "NEAR");
        assert_eq!(portfolio.risk_tolerance, 5);
    }

    #[test]
    fn test_ai_recommendation() {
        let (mut contract, mut context) = setup();
        let rec = r#"{"buy":"NEAR","amount":5}"#.to_string();
        contract.set_ai_recommendation(rec.clone());
        assert_eq!(contract.get_ai_recommendation(), Some(rec));
    }

    #[test]
    fn test_trade_request() {
        let (mut contract, mut context) = setup();
        testing_env!(context.attached_deposit(1_000_000_000_000).build()); // 1 NEAR
        contract.request_trade("ETH".to_string(), 2_000_000_000_000, "buy".to_string());
        let history = contract.get_trade_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].asset, "ETH");
        assert_eq!(history[0].action, "buy");
    }
}