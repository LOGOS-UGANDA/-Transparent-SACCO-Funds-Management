import "@nomicfoundation/hardhat-toolbox";

/** @type import('hardhat/config').HardhatUserConfig */
export default {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: { enabled: true, runs: 200 },
    },
  },
  networks: {
    localhost: {
      url: "http://127.0.0.1:8545",
    },
    logos: {
      url: process.env.LOGOS_RPC_URL || "https://rpc.logos.co",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      chainId: parseInt(process.env.LOGOS_CHAIN_ID || "0"),
    },
  },
  paths: {
    sources: "./contracts",
    artifacts: "./src/artifacts",
  },
};
