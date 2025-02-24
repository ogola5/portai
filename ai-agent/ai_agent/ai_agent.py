import json
from near_api import Near
from near_api.account import Account
from near_api.providers import JsonProvider
import pandas as pd
from sklearn.linear_model import LinearRegression
import requests
import os

# NEAR setup
NEAR_NODE_URL = "https://rpc.testnet.near.org"
WALLET_KEY = "YOUR_PRIVATE_KEY"  # From NEAR testnet wallet
CONTRACT_ID = "dev-1234-5678"    # Replace with your deployed NEAR contract ID
near = Near(JsonProvider(NEAR_NODE_URL))
account = Account(near, WALLET_KEY, "yourname.testnet")

# Mock on-chain data fetch (replace with real NEAR Indexer later)
def fetch_portfolio(account_id):
    result = account.view_function(CONTRACT_ID, "get_portfolio", {"account_id": account_id})
    return result if result else {"assets": [], "risk_tolerance": 5}

def fetch_trade_history(account_id):
    result = account.view_function(CONTRACT_ID, "get_trade_history", {"account_id": account_id})
    return result if result else []

# Portfolio Analyzer Agent
def portfolio_analyzer(account_id):
    portfolio = fetch_portfolio(account_id)
    assets = pd.DataFrame(portfolio["assets"])
    if assets.empty:
        return {"recommendation": "No assets to analyze"}

    # Mock price data (replace with real API, e.g., CoinGecko)
    price_data = {
        "NEAR": [10, 10.2, 10.1, 10.5, 10.3],
        "ETH": [2000, 2050, 2020, 2100, 2080]
    }
    risk_tolerance = portfolio["risk_tolerance"]

    # Simple regression to predict next price
    recommendations = {}
    for _, row in assets.iterrows():
        asset = row["name"]
        if asset in price_data:
            X = pd.DataFrame({"time": range(len(price_data[asset]))})
            y = price_data[asset]
            model = LinearRegression()
            model.fit(X, y)
            next_price = model.predict([[len(price_data[asset])]])[0]
            current_price = y[-1]
            
            # Suggest based on risk tolerance (1=conservative, 10=aggressive)
            if next_price > current_price and risk_tolerance > 3:
                recommendations[asset] = {"action": "buy", "amount": int(1000 / current_price)}
            elif next_price < current_price and risk_tolerance < 7:
                recommendations[asset] = {"action": "sell", "amount": int(row["amount"] * 0.1)}

    return {"recommendation": json.dumps(recommendations)}

# Trade Execution Agent
def trade_execution(account_id):
    recommendation = account.view_function(CONTRACT_ID, "get_ai_recommendation", {"account_id": account_id})
    if not recommendation:
        return "No recommendation to execute"

    rec = json.loads(recommendation)
    portfolio = fetch_portfolio(account_id)
    risk_tolerance = portfolio["risk_tolerance"]

    for asset, trade in rec.items():
        action = trade["action"]
        amount = trade["amount"] * 10**24  # Convert to yoctoNEAR scale (mock scaling)
        if (action == "buy" and risk_tolerance > 3) or (action == "sell" and risk_tolerance < 7):
            # Call NEAR contract to request trade
            account.function_call(
                CONTRACT_ID,
                "request_trade",
                {"asset": asset, "amount": amount, "action": action},
                gas=30000000000000,  # 30 TGas
                amount=100000000000000000000000  # 0.1 NEAR for gas
            )
            print(f"Executed trade: {action} {amount} {asset}")
    return "Trades executed"

# Liquidity Optimizer
def liquidity_optimizer(account_id):
    # Mock liquidity pool data (replace with real API, e.g., Ref Finance or Trisolaris)
    pools = [
        {"pair": "NEAR-USDT", "apy": 12.5, "tvl": 1000000},
        {"pair": "ETH-USDT", "apy": 8.0, "tvl": 500000},
        {"pair": "NEAR-ETH", "apy": 15.0, "tvl": 200000}
    ]
    
    # Simple scoring: APY * (1 - risk_factor), risk_factor based on TVL
    portfolio = fetch_portfolio(account_id)
    risk_tolerance = portfolio["risk_tolerance"]
    best_pool = max(pools, key=lambda p: p["apy"] * (1 - (1 / (p["tvl"] / 1000000))) if p["tvl"] > 100000 else 0)
    
    if risk_tolerance <= 5 and best_pool["apy"] < 10:
        return {"recommendation": "No safe staking options"}
    return {"recommendation": json.dumps({"stake": best_pool["pair"], "apy": best_pool["apy"]})}

# Main execution
def run_ai_agents(account_id):
    # Portfolio Analyzer
    analyzer_result = portfolio_analyzer(account_id)
    account.function_call(
        CONTRACT_ID,
        "set_ai_recommendation",
        {"recommendation": analyzer_result["recommendation"]},
        gas=30000000000000
    )
    print(f"Analyzer: {analyzer_result}")

    # Trade Execution
    execution_result = trade_execution(account_id)
    print(f"Execution: {execution_result}")

    # Liquidity Optimizer
    optimizer_result = liquidity_optimizer(account_id)
    account.function_call(
        CONTRACT_ID,
        "set_ai_recommendation",
        {"recommendation": optimizer_result["recommendation"]},
        gas=30000000000000
    )
    print(f"Optimizer: {optimizer_result}")

if __name__ == "__main__":
    USER_ACCOUNT = "yourname.testnet"  # Replace with your testnet account
    run_ai_agents(USER_ACCOUNT)