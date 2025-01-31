// const StorageStaking = artifacts.require("StorageStaking");

// module.exports = function(deployer) {
//   const stakingTokenAddress = "0x55d398326f99059fF775485246999027B3197955";  // Buraya staking token adresinizi ekleyin
//   deployer.deploy(StorageStaking, stakingTokenAddress);
// };
const MyToken = artifacts.require("MyToken");
const StorageStaking = artifacts.require("StorageStaking");

module.exports = async function (deployer, network, accounts) {
  // ERC20 token'ı deploy ediyoruz (örneğin 1 milyon token)
  const initialSupply = web3.utils.toWei('1000000', 'ether'); // 1 milyon token mint eder

  await deployer.deploy(MyToken, initialSupply);  // ERC20 token'ı deploy et

  // Token'ın adresini alıyoruz
  const myToken = await MyToken.deployed();
  const tokenAddress = myToken.address;

  // StorageStaking sözleşmesini deploy ediyoruz ve token adresini constructor'a veriyoruz
  await deployer.deploy(StorageStaking, tokenAddress);
  
};
