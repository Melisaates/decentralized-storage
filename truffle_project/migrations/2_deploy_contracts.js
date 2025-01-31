const StorageStaking = artifacts.require("StorageStaking");

module.exports = function(deployer) {
  const stakingTokenAddress = "0xYourTokenAddress";  // Buraya staking token adresinizi ekleyin
  deployer.deploy(StorageStaking, stakingTokenAddress);
};
