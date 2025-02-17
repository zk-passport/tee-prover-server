import * as ethers from "ethers";
import { Pcr0Manager__factory } from "./typechain/factories";

async function main() {
  const address = process.argv[2];
  const privateKey = process.argv[3];
  const pcr0 = process.argv[4];

  if (!address || !privateKey || !pcr0) {
    console.log(
      "Usage: tsx index.ts <address> <private key> <pcr0 measurement>"
    );
    process.exit(1);
  }

  const provider = new ethers.JsonRpcProvider("https://rpc.ankr.com/celo");
  const wallet = new ethers.Wallet(privateKey, provider);

  const factory = Pcr0Manager__factory.connect(address, wallet);

  await factory.addPCR0(pcr0);
}
