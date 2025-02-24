require("@nomicfoundation/hardhat-toolbox");

module.exports = {
  solidity: "0.8.20",
  networks: {
    aurora_testnet: {
      url: "https://testnet.aurora.dev",
      accounts: ["YOUR_PRIVATE_KEY"], // From NEAR Wallet, export private key
      chainId: 1313161555,
    },
  },
};