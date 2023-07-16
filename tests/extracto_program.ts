import { expect } from "chai";
import { PublicKey, SystemProgram, } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ExtractoProgram } from "../target/types/extracto_program";
import { getAccount } from "@solana/spl-token";
import { spawn } from "child_process";
import { assert } from "chai";

// 0️⃣  Import the Clockwork SDK.
import { ClockworkProvider } from "@clockwork-xyz/sdk";

const print_address = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/address/${address}?cluster=devnet`);
}

const print_thread = async (clockworkProvider, address) => {
  const threadAccount = await clockworkProvider.getThreadAccount(address);
  console.log("\nThread: ", threadAccount, "\n");
  print_address("🧵 Thread", address);
  console.log("\n")
}

const print_tx = (label, address) => {
  console.log(`${label}: https://explorer.solana.com/tx/${address}?cluster=devnet`);
}

const stream_program_logs = (programId) => {
  const cmd = spawn("solana", ["logs", "-u", "devnet", programId.toString()]);
    cmd.stdout.on("data", data => {
        console.log(`Program Logs: ${data}`);
    });
}

const verifyAmount = async (connection, ata, expectedAmount) => {
  const amount = (await getAccount(connection, ata)).amount;
  assert.equal(amount.toString(), expectedAmount.toString());
  return amount;
}

let lastThreadExec = BigInt(0);
const waitForThreadExec = async (clockworkProvider, thread: PublicKey, maxWait: number = 60) => {
  let i = 1;
  while (true) {
      const execContext = (await clockworkProvider.getThreadAccount(thread)).execContext;
      if (execContext) {
          if (lastThreadExec.toString() == "0" || execContext.lastExecAt > lastThreadExec) {
              lastThreadExec = execContext.lastExecAt;
              break;
          }
      }
      if (i == maxWait) throw Error("Timeout");
      i += 1;
      await new Promise((r) => setTimeout(r, i * 1000));
  }
}


const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const wallet = provider.wallet;
const program = anchor.workspace.ExtractoProgram as Program<ExtractoProgram>;
const clockworkProvider = ClockworkProvider.fromAnchorProvider(provider);


/*
** Helpers
*/
const fetchCounter = async (counter) => {
    const counterAcc = await program.account.counter.fetch(counter);
    console.log("currentValue: " + counterAcc.currentValue + ", updatedAt: " + counterAcc.updatedAt);
    return counterAcc;
}


/*
** Tests
*/
describe("extracto_program", () => {
    print_address("🤖 Counter program", program.programId.toString());
    const [counter] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("counter")], // 👈 make sure it matches on the prog side
        program.programId
    );

    // 1️⃣ Prepare thread address
    const threadId = "counter-" + new Date().getTime() / 1000;
    const [threadAuthority] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("authority")], // 👈 make sure it matches on the prog side
        program.programId
    );
    const [threadAddress, threadBump] = clockworkProvider.getThreadPDA(threadAuthority, threadId)

    it("It increments every 10 seconds", async () => {
        try {
            // 2️⃣ Ask our program to create a thread via CPI
            // and thus become the admin of that thread
            await program.methods
                .initialize(Buffer.from(threadId))
                .accounts({
                    payer: wallet.publicKey,
                    systemProgram: SystemProgram.programId,
                    clockworkProgram: clockworkProvider.threadProgram.programId,
                    thread: threadAddress,
                    threadAuthority: threadAuthority,
                    counter: counter,
                })
                .rpc();
            await print_thread(clockworkProvider, threadAddress);

            console.log("Verifying that Thread increments the counter every 10s")
            for (let i = 1; i < 4; i++) {
                await waitForThreadExec(clockworkProvider, threadAddress);
                const counterAcc = await fetchCounter(counter);
                expect(counterAcc.currentValue.toString()).to.eq(i.toString());
            }
        } catch (e) {
            // ❌
            // 'Program log: Instruction: ThreadCreate',
            // 'Program 11111111111111111111111111111111 invoke [2]',
            // 'Allocate: account Address { address: ..., base: None } already in use'
            //
            // -> If you encounter this error, the thread address you are trying to use is already in use.
            //    You can change the threadId, to generate a new account address.
            // -> OR update the thread with a ThreadUpdate instruction (more on this in future guide)
            console.error(e);
            expect.fail(e);
        }
    })

    // Just some cleanup to reset the test to a clean state
    afterEach(async () => {
        try {
            await program.methods
                .reset()
                .accounts({
                    payer: wallet.publicKey,
                    clockworkProgram: clockworkProvider.threadProgram.programId,
                    counter: counter,
                    thread: threadAddress,
                    threadAuthority: threadAuthority,
                })
                .rpc();
        } catch (e) { }
    })
});
