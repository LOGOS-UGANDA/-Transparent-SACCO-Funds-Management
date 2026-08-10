import hre from "hardhat";

async function main() {
  console.log("Deploying FeeTransparency to", hre.network.name, "...\n");

  const FeeTransparency = await hre.ethers.getContractFactory("FeeTransparency");
  const contract = await FeeTransparency.deploy();
  await contract.waitForDeployment();

  const address = await contract.getAddress();
  console.log("FeeTransparency deployed to:", address);

  if (hre.network.name === "localhost") {
    console.log("\nSeeding demo data...\n");
    const [owner, approver1] = await hre.ethers.getSigners();

    const tx1 = await contract.registerInstitution(
      "Makerere University",
      "MUK-2024-001",
      0, // University
      owner.address
    );
    await tx1.wait();
    console.log("  Registered: Makerere University (ID: 1)");

    const tx2 = await contract.registerInstitution(
      "Victoria University",
      "VU-2024-002",
      0,
      owner.address
    );
    await tx2.wait();
    console.log("  Registered: Victoria University (ID: 2)");

    const tx3 = await contract.registerInstitution(
      "Kampala Teachers SACCO",
      "KTS-2024-003",
      1, // SACCO
      owner.address
    );
    await tx3.wait();
    console.log("  Registered: Kampala Teachers SACCO (ID: 3)");

    await (await contract.verifyInstitution(1)).wait();
    await (await contract.verifyInstitution(2)).wait();
    await (await contract.verifyInstitution(3)).wait();
    console.log("  Verified institutions 1, 2, 3");

    const makCategories = [
      ["Tuition Fees", "Academic instruction and course delivery", 4000],
      ["Library & Research", "Library resources and research funding", 1500],
      ["ICT Infrastructure", "Computer labs and digital systems", 1000],
      ["Sports & Recreation", "Sports facilities and recreation", 800],
      ["Medical Services", "University health center", 700],
      ["Student Guild", "Student governance and activities", 500],
      ["Infrastructure Dev", "Building maintenance and construction", 1500],
    ];

    for (const [name, desc, pct] of makCategories) {
      await (await contract.createFeeCategory(1, name, desc, pct)).wait();
      console.log(`  Category: ${name} (${pct / 100}%)`);
    }

    if (approver1) {
      await (await contract.addApprover(1, approver1.address)).wait();
      console.log("  Added approver for Makerere:", approver1.address);
    }

    console.log("\nDeployment and seeding complete!");
    console.log("Contract address:", address);
    console.log("\nAdd this to your .env file:");
    console.log(`VITE_CONTRACT_ADDRESS=${address}`);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
