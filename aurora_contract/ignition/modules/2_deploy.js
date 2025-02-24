const hre = require("hardhat");

async function main() {
  const PortfolioTrader = await hre.ethers.getContractFactory("PortfolioTrader");
  const trader = await PortfolioTrader.deploy();
  await trader.deployed();
  console.log("PortfolioTrader deployed to:", trader.address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});