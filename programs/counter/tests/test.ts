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
    lamports
} from "@solana/web3.js";
import * as crypto from "crypto";
import path from "path";

function getAnchorDiscriminator(prefix: string, name: string): Uint8Array {
    const hash = crypto.createHash("sha256").update(`${prefix}:${name}`).digest();
    return new Uint8Array(hash.subarray(0, 8));
}

async function runBenchmark() {
    const svm = new LiteSVM();

    const mockPayer = await generateKeyPairSigner();
    const nativeCounterAccount = await generateKeyPairSigner();
    const pinocchioCounterAccount = await generateKeyPairSigner();
    const anchorCounterAccount = await generateKeyPairSigner();

    const nativeProgramId = (await generateKeyPairSigner()).address as any;
    const pinocchioProgramId = (await generateKeyPairSigner()).address as any;
    const anchorProgramId = address("ChT1pY9D9Db9jG7FmG7FmG7FmG7FmG7FmG7FmG7FmG7F") as any;

    svm.airdrop(mockPayer.address as any, lamports(1_000_000_000n) as any);

    svm.addProgramFromFile(nativeProgramId, path.resolve(__dirname, "../../../target/deploy/counter_native.so"));
    svm.addProgramFromFile(pinocchioProgramId, path.resolve(__dirname, "../../../target/deploy/counter_pinocchio.so"));
    svm.addProgramFromFile(anchorProgramId, path.resolve(__dirname, "../../../target/deploy/counter_anchor.so"));

    const anchorAccountDisc = getAnchorDiscriminator("account", "Counter");
    const anchorIxDisc = getAnchorDiscriminator("global", "increment");
    const anchorInitialData = new Uint8Array(16);
    anchorInitialData.set(anchorAccountDisc, 0);

    svm.setAccount({
        address: nativeCounterAccount.address as any,
        lamports: lamports(2_000_000n) as any,
        data: new Uint8Array(8),
        programAddress: nativeProgramId,
        executable: false,
        space: 8n
    } as any);

    svm.setAccount({
        address: pinocchioCounterAccount.address as any,
        lamports: lamports(2_000_000n) as any,
        data: new Uint8Array(8),
        programAddress: pinocchioProgramId,
        executable: false,
        space: 8n
    } as any);

    svm.setAccount({
        address: anchorCounterAccount.address as any,
        lamports: lamports(2_000_000n) as any,
        data: anchorInitialData,
        programAddress: anchorProgramId,
        executable: false,
        space: 16n
    } as any);

    const nativeInstruction = {
        programAddress: nativeProgramId,
        accounts: [{ address: nativeCounterAccount.address, role: AccountRole.WRITABLE }],
        data: new Uint8Array(0),
    };

    const nativeTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => (svm as any).setTransactionMessageLifetimeUsingLatestBlockhash(tx),
        (tx) => appendTransactionMessageInstruction(nativeInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx),
    );

    const nativeResult = svm.sendTransaction(nativeTx as any) as any;
    const nativeLogs = typeof nativeResult.logs === "function" ? nativeResult.logs() : nativeResult.logs;
    if (nativeLogs) {
        console.log(`\nNative: ${nativeLogs.find((m: string) => m.includes("consumed"))}`);
    }

    const pinocchioInstruction = {
        programAddress: pinocchioProgramId,
        accounts: [{ address: pinocchioCounterAccount.address, role: AccountRole.WRITABLE }],
        data: new Uint8Array(0),
    };

    const pinocchioTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => (svm as any).setTransactionMessageLifetimeUsingLatestBlockhash(tx),
        (tx) => appendTransactionMessageInstruction(pinocchioInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx),
    );

    const pinocchioResult = svm.sendTransaction(pinocchioTx as any) as any;
    const pinocchioLogs = typeof pinocchioResult.logs === "function" ? pinocchioResult.logs() : pinocchioResult.logs;
    if (pinocchioLogs) {
        console.log(`Pinocchio: ${pinocchioLogs.find((m: string) => m.includes("consumed"))}`);
    }

    const anchorInstruction = {
        programAddress: anchorProgramId,
        accounts: [{ address: anchorCounterAccount.address, role: AccountRole.WRITABLE }],
        data: anchorIxDisc,
    };

    const anchorTx = await pipe(
        createTransactionMessage({ version: 0 }),
        (tx) => setTransactionMessageFeePayerSigner(mockPayer, tx),
        (tx) => (svm as any).setTransactionMessageLifetimeUsingLatestBlockhash(tx),
        (tx) => appendTransactionMessageInstruction(anchorInstruction as any, tx),
        (tx) => signTransactionMessageWithSigners(tx),
    );

    const anchorResult = svm.sendTransaction(anchorTx as any) as any;
    const anchorLogs = typeof anchorResult.logs === "function" ? anchorResult.logs() : anchorResult.logs;
    if (anchorLogs) {
        console.log(`Anchor: ${anchorLogs.find((m: string) => m.includes("consumed"))}`);
    } else {
        console.error("Anchor Execution Failed:", anchorResult);
    }
}

runBenchmark().catch(console.error);