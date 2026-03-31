/**
 * privatemed-solana demo
 * Encrypted medication interaction check via Arcium MXE
 *
 * Usage:
 *   ANCHOR_WALLET=~/.config/solana/devnet.json npx ts-node --transpile-only scripts/run_demo.ts
 */
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { randomBytes } from "crypto";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import {
  awaitComputationFinalization,
  getArciumEnv,
  getCompDefAccOffset,
  RescueCipher,
  deserializeLE,
  getMXEPublicKey,
  getMXEAccAddress,
  getMempoolAccAddress,
  getCompDefAccAddress,
  getExecutingPoolAccAddress,
  getComputationAccAddress,
  getClusterAccAddress,
  x25519,
} from "@arcium-hq/client";

const PROGRAM_ID = new PublicKey("4VxzAxbQspysvUAvRdkdNxw9Ew3LZaAP2V78gMrQirUY");
const EVIDENCE_LOG = path.join(__dirname, "../evidence/mxe_runs.jsonl");

function log(event: string, data: Record<string, unknown> = {}) {
  const line = JSON.stringify({ event, ...data, ts: new Date().toISOString() });
  fs.mkdirSync(path.dirname(EVIDENCE_LOG), { recursive: true });
  fs.appendFileSync(EVIDENCE_LOG, line + "\n");
  console.log(line);
}

async function getMxePublicKeyWithRetry(
  provider: anchor.AnchorProvider,
  programId: PublicKey,
  retries = 5,
  delayMs = 1000,
): Promise<Uint8Array> {
  for (let attempt = 1; attempt <= retries; attempt++) {
    const key = await getMXEPublicKey(provider, programId);
    if (key) {
      return key;
    }
    if (attempt < retries) {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }
  throw new Error(`MXE public key unavailable for program ${programId.toString()}`);
}

async function main() {
  process.env.ARCIUM_CLUSTER_OFFSET = "456";

  const walletPath = process.env.ANCHOR_WALLET || `${os.homedir()}/.config/solana/devnet.json`;
  const conn = new anchor.web3.Connection(
    process.env.ANCHOR_PROVIDER_URL || "https://api.devnet.solana.com",
    "confirmed",
  );
  const owner = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(walletPath).toString())),
  );
  const provider = new anchor.AnchorProvider(conn, new anchor.Wallet(owner), {
    commitment: "confirmed",
    skipPreflight: true,
  });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(path.join(__dirname, "../target/idl/privatemed.json"), "utf-8"));
  const program = new anchor.Program(idl, provider) as anchor.Program<any>;
  const arciumEnv = getArciumEnv();

  log("demo_start", {
    program: PROGRAM_ID.toString(),
    wallet: owner.publicKey.toString(),
    description: "Encrypted medication interaction check via MXE",
  });

  const privateKey = x25519.utils.randomSecretKey();
  const publicKey = x25519.getPublicKey(privateKey);
  const mxePublicKey = await getMxePublicKeyWithRetry(provider, PROGRAM_ID);

  const drug1 = BigInt(Math.floor(Math.random() * 100) + 1);
  const drug2 = BigInt(Math.floor(Math.random() * 100) + 1);
  log("medication_pair", {
    drug1: "encrypted",
    drug2: "encrypted",
    note: `Local sample codes prepared for private interaction check (${drug1.toString()}, ${drug2.toString()})`,
  });

  const nonce = randomBytes(16);
  const sharedSecret = x25519.getSharedSecret(privateKey, mxePublicKey);
  const cipher = new RescueCipher(sharedSecret);
  const ciphertext = cipher.encrypt([drug1, drug2], nonce);

  const computationOffset = new anchor.BN(randomBytes(8), "hex");
  const clusterOffset = arciumEnv.arciumClusterOffset;

  try {
    const sig = await program.methods
      .checkInteraction(
        computationOffset,
        Array.from(ciphertext[0]),
        Array.from(ciphertext[1]),
        Array.from(publicKey),
        new anchor.BN(deserializeLE(nonce).toString()),
      )
      .accountsPartial({
        payer: owner.publicKey,
        mxeAccount: getMXEAccAddress(PROGRAM_ID),
        mempoolAccount: getMempoolAccAddress(clusterOffset),
        executingPool: getExecutingPoolAccAddress(clusterOffset),
        computationAccount: getComputationAccAddress(clusterOffset, computationOffset),
        compDefAccount: getCompDefAccAddress(
          PROGRAM_ID,
          Buffer.from(getCompDefAccOffset("check_interaction")).readUInt32LE(),
        ),
        clusterAccount: getClusterAccAddress(clusterOffset),
      })
      .rpc({ skipPreflight: true, commitment: "confirmed" });

    log("interaction_queued", {
      sig,
      explorer: `https://explorer.solana.com/tx/${sig}?cluster=devnet`,
      note: "Medication interaction check queued in MXE cluster 456",
    });

    const finalizeSig = await Promise.race([
      awaitComputationFinalization(provider, computationOffset, PROGRAM_ID, "confirmed"),
      new Promise<never>((_, reject) => setTimeout(() => reject(new Error("timeout")), 90_000)),
    ]);

    log("interaction_success", {
      queueSig: sig,
      finalizeSig,
      clusterOffset,
    });
  } catch (e: any) {
    log("interaction_fail", {
      message: e.message || String(e),
      logs: e.logs || [],
      code: e.code,
      raw: (() => { try { return JSON.stringify(e); } catch { return String(e); } })(),
    });
    process.exit(1);
  }
}

main().catch((e) => {
  console.error(JSON.stringify({ event: "fatal", message: e.message }));
  process.exit(1);
});
