import { LiteSVM } from "litesvm";
import {
    address,
    generateKeyPairSigner,
    pipe,
    createTransactionMessage,
    setTransactionMessageFeePayerSigner,
    appendTransactionMessageInstruction,
    signTransactionMessageWithSigners,
    AccountRole,
    lamports,
    Address
} from "@solana/web3.js";
import * as crypto from "crypto";
import * as path from "path";

function serializeAddressInfo(addressName: string, houseNumber: number, street: string, city: string): Uint8Array {
    const encodeString = (str: string) => {
        const buf = Buffer.from(str, 'utf-8');
        const lenBuf = Buffer.alloc(4);
        lenBuf.writeUInt32LE(buf.length, 0);
        return Buffer.concat([lenBuf, buf]);
    };

    const nameBuf = encodeString(addressName);
    const houseBuf = Buffer.from([houseNumber]);
    const streetBuf = encodeString(street);
    const cityBuf = encodeString(city);

    return new Uint8Array(Buffer.concat([nameBuf, houseBuf, streetBuf, cityBuf]));
}

async function runTest() {
    const svm = new LiteSVM();

    const mockPayer = await generateKeyPairSigner();
    const addressInfoAccount = await generateKeyPairSigner();

    const programId = (await generateKeyPairSigner()).address as any;
    const systemProgramId = address("11111111111111111111111111111111");

    svm.airdrop(mockPayer.address as any, lamports(1_000_000_000n) as any);

    svm.addProgramFromFile(programId, path.resolve(__dirname, "../../../target/deploy/account_data_native.so"));

    console.log("Serializing address.....");
    const testPayload = serializeAddressInfo(
        "Solana Foundation",
        108,
        "DeFi Boulevard",
        "Crypto City"
    );
    console.log("Address serialized");

    const createAddressInstruction = {
        programAddress: programId,
        accounts: [
            { address: addressInfoAccount.address, role: AccountRole.WRITABLE_SIGNER, signer: addressInfoAccount },
            { address: mockPayer.address, role: AccountRole.WRITABLE_SIGNER },
            { address: systemProgramId, role: AccountRole.READONLY }
        ],
        data: testPayload
    };

    const txMessage = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => (svm as any).setTransactionMessageLifetimeUsingLatestBlockhash(tx),
        (tx) => appendTransactionMessageInstruction(createAddressInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx)
    );

    console.log("Executing account creation...");
    const result = svm.sendTransaction(txMessage as any);
    console.log("Result: ", result);

    /* const logs = typeof result.logs === "function" ? result.logs() : result.logs;
    if (logs) {
        console.log("\n=================== RUNTIME LOGS ===================");
        console.log(logs.join("\n"));
        console.log("====================================================");
    } */

    const fetchedAccount = svm.getAccount(addressInfoAccount.address as any);
    console.log("Fetch account: ", fetchedAccount);
}

runTest().catch(console.error);