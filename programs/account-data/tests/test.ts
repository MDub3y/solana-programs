import {
    address,
    createSolanaRpc,
    generateKeyPairSigner,
    pipe,
    createTransactionMessage,
    setTransactionMessageFeePayerSigner,
    appendTransactionMessageInstruction,
    signTransactionMessageWithSigners,
    getSignatureFromTransaction,
    getBase64EncodedWireTransaction,
    AccountRole,
    lamports
} from "@solana/web3.js";
import * as crypto from "crypto";

function getAnchorIxDiscriminator(prefix: string, name: string): Buffer {
    const hash = crypto.createHash("sha256").update(`${prefix}:${name}`).digest();
    return hash.subarray(0, 8);
}

function serializeAddressInfo(addressName: string, houseNumber: number, street: string, city: string): Buffer {
    const encodeString = (str: string) => {
        const buf = Buffer.from(str, 'utf-8');
        const lenBuf = Buffer.alloc(4);
        lenBuf.writeUInt32LE(buf.length, 0);
        return Buffer.concat([lenBuf, buf]);
    };
    return Buffer.concat([encodeString(addressName), Buffer.from([houseNumber]), encodeString(street), encodeString(city)]);
}

function serializePinocchioFixedLayout(name: string, houseNumber: number, street: string, city: string): Uint8Array {
    const payload = new Uint8Array(51);
    const encoder = new TextEncoder();
    payload.set(encoder.encode(name).subarray(0, 16), 0);
    payload[16] = houseNumber & 0xFF;
    payload.set(encoder.encode(street).subarray(0, 16), 17);
    payload.set(encoder.encode(city).subarray(0, 18), 33);
    return payload;
}

async function runTest() {
    const rpc = createSolanaRpc('http://127.0.0.1:8899');
    const systemProgramId = address("11111111111111111111111111111111");

    const nativeProgramId = address("6tZ4GENA2BUzsKSBWoW6Ci9mXPDaQ2oyiyJqY9vHcvcW");
    const anchorProgramId = address("9KEFAFYPhcFQDELpCVHkuezsUE3iAZjPNsdkKA8UHMa2");
    const pinocchioProgramId = address("9wofj1sapib3LnjbE8ByGw5TWks8PqUk4jtiUK5DUBVp");

    const nativeStateAccount = await generateKeyPairSigner();
    const anchorStateAccount = await generateKeyPairSigner();
    const pinocchioStateAccount = await generateKeyPairSigner();

    const mockPayer = await generateKeyPairSigner();
    console.log("Payer address generated: " + mockPayer.address);

    console.log("Requesting execution capital from your local faucet...");
    await rpc.requestAirdrop(mockPayer.address, lamports(2_000_000_000n)).send();

    console.log("Awaiting airdrop confirmation...");
    let balance = 0n;
    while (balance === 0n) {
        const accountInfo = await rpc.getAccountInfo(mockPayer.address).send();
        balance = accountInfo.value?.lamports || 0n;
    }
    console.log("Payer funded! Balance: " + balance + " lamports");

    const name = "Solana HQ";
    const houseNumber = 77;
    const street = "Validator Lane";
    const city = "Decentralized Zone";

    const rawBorshPayload = serializeAddressInfo(name, houseNumber, street, city);

    const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();
    const blockhashConstraint = {
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
    };

    console.log("\nPreparing Native transaction packet...");

    const nativeInstruction = {
        programAddress: nativeProgramId,
        accounts: [
            { address: nativeStateAccount.address, role: AccountRole.WRITABLE_SIGNER, signer: nativeStateAccount },
            { address: mockPayer.address, role: AccountRole.WRITABLE_SIGNER, signer: mockPayer },
            { address: systemProgramId, role: AccountRole.READONLY }
        ],
        data: new Uint8Array(rawBorshPayload)
    };

    const nativeTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => ({ ...tx, lifetimeConstraint: blockhashConstraint }),
        (tx) => appendTransactionMessageInstruction(nativeInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx)
    );

    const nativeSig = getSignatureFromTransaction(nativeTx);
    const serializedNativeWire = getBase64EncodedWireTransaction(nativeTx);
    await rpc.sendTransaction(serializedNativeWire, { encoding: 'base64' }).send();

    console.log("NATIVE SIGNATURE: " + nativeSig);
    console.log("Native Storage Key: " + nativeStateAccount.address);

    console.log("\nPreparing Anchor transaction packet...");

    const anchorIxDisc = getAnchorIxDiscriminator("global", "create_address_info");
    const anchorPayload = Buffer.concat([anchorIxDisc, rawBorshPayload]);

    const anchorInstruction = {
        programAddress: anchorProgramId,
        accounts: [
            { address: mockPayer.address, role: AccountRole.WRITABLE_SIGNER, signer: mockPayer },
            { address: anchorStateAccount.address, role: AccountRole.WRITABLE_SIGNER, signer: anchorStateAccount },
            { address: systemProgramId, role: AccountRole.READONLY }
        ],
        data: new Uint8Array(anchorPayload)
    };

    const anchorTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => ({ ...tx, lifetimeConstraint: blockhashConstraint }),
        (tx) => appendTransactionMessageInstruction(anchorInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx)
    );

    const anchorSig = getSignatureFromTransaction(anchorTx);
    const serializedAnchorWire = getBase64EncodedWireTransaction(anchorTx);
    await rpc.sendTransaction(serializedAnchorWire, { encoding: 'base64' }).send();

    console.log("ANCHOR SIGNATURE: " + anchorSig);
    console.log("Anchor Storage Key: " + anchorStateAccount.address);

    console.log("\nPreparing Pinocchio transaction packet...");

    const fixedDataPayload = serializePinocchioFixedLayout(name, houseNumber, street, city);
    const pinocchioPayload = new Uint8Array(1 + fixedDataPayload.length);
    pinocchioPayload[0] = 0;
    pinocchioPayload.set(fixedDataPayload, 1);

    const pinocchioInstruction = {
        programAddress: pinocchioProgramId,
        accounts: [
            { address: pinocchioStateAccount.address, role: AccountRole.WRITABLE_SIGNER, signer: pinocchioStateAccount },
            { address: mockPayer.address, role: AccountRole.WRITABLE_SIGNER, signer: mockPayer },
            { address: systemProgramId, role: AccountRole.READONLY }
        ],
        data: pinocchioPayload
    };

    const pinocchioTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => ({ ...tx, lifetimeConstraint: blockhashConstraint }),
        (tx) => appendTransactionMessageInstruction(pinocchioInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx)
    );

    const pinocchioSig = getSignatureFromTransaction(pinocchioTx);
    const serializedPinocchioWire = getBase64EncodedWireTransaction(pinocchioTx);
    await rpc.sendTransaction(serializedPinocchioWire, { encoding: 'base64' }).send();

    console.log("PINOCCHIO SIGNATURE: " + pinocchioSig);
    console.log("Pinocchio Storage Key: " + pinocchioStateAccount.address + "\n");
}

runTest().catch(console.error);