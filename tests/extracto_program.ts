import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { expect } from "chai"
import { ExtractoProgram } from "../target/types/extracto_program"
import { PublicKey, SystemProgram, } from "@solana/web3.js";
import { getAccount } from "@solana/spl-token";

describe("extracto-program", () => {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(provider)
    const program = anchor.workspace.ExtractoProgram as Program<ExtractoProgram>

    const [counterAddress] = PublicKey.findProgramAddressSync(
        [anchor.utils.bytes.utf8.encode("counter_1")], // 👈 make sure it matches on the prog side
        program.programId
    );

    it("Is initialized!", async () => {

        let counterAccount

        try{
            counterAccount = await program.account.counter.fetch(counterAddress)
            console.log("counterAccount is already initialized");
        }
        catch
        {
            const txHash = await program.methods
            .initialize()
            .accounts({ 
                counter: counterAddress,
            })
            .rpc()
            counterAccount = await program.account.counter.fetch(counterAddress)
        }

        expect(counterAccount.count.toNumber() === 0)
    })

    it("Incremented the count", async () => {
        const tx = await program.methods
            .increment()
            .accounts({ counter: counterAddress, user: provider.wallet.publicKey })
            .rpc()

        const account = await program.account.counter.fetch(counterAddress)
        expect(account.count.toNumber() === 1)
    })

    it("Reset the count", async () => {
        const tx = await program.methods
            .reset()
            .accounts({ counter: counterAddress, user: provider.wallet.publicKey })
            .rpc()

        const account = await program.account.counter.fetch(counterAddress)
        expect(account.count.toNumber() === 0)
    })
})
