import {
    Keypair,
    PublicKey,
    SystemProgram,
    TransactionInstruction,
} from "@solana/web3.js";
import {
    getVirtualPool,
    processTransactionMaybeThrow,
    VirtualCurveProgram,
    getConfig,
    derivePoolAuthority,
    deriveBaseKeyForLocker,
    LOCKER_PROGRAM_ID,
    getOrCreateAssociatedTokenAccount,
} from "../utils";
import { BanksClient } from "solana-bankrun";
import {
    TOKEN_2022_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export type CreateLockerParameters = {
    payer: Keypair;
    virtualPool: PublicKey;
};

export async function createLocker(
    banksClient: BanksClient,
    program: VirtualCurveProgram,
    params: CreateLockerParameters
): Promise<any> {
    const { payer, virtualPool } = params;
    const virtualPoolState = await getVirtualPool(
        banksClient,
        program,
        virtualPool
    );
    const configState = await getConfig(banksClient, program, virtualPoolState.config);
    const base = deriveBaseKeyForLocker(virtualPool);
    const escrow = deriveLockerEscrow(base);
    const tokenProgram =
        configState.tokenType == 0 ? TOKEN_PROGRAM_ID : TOKEN_2022_PROGRAM_ID;
    const preInstructions: TransactionInstruction[] = [];
    const [{ ata: escrowToken, ix: createOwnerEscrowVaultTokenXIx }] = await Promise.all([getOrCreateAssociatedTokenAccount(
        banksClient,
        payer,
        virtualPoolState.baseMint,
        escrow,
        tokenProgram
    )]);

    createOwnerEscrowVaultTokenXIx && preInstructions.push(createOwnerEscrowVaultTokenXIx);

    const transaction = await program.methods
        .createLocker()
        .accountsPartial({
            virtualPool,
            config: virtualPoolState.config,
            poolAuthority: derivePoolAuthority(),
            baseVault: virtualPoolState.baseVault,
            baseMint: virtualPoolState.baseMint,
            base,
            creator: virtualPoolState.creator,
            escrow,
            escrowToken,
            payer: payer.publicKey,
            tokenProgram,
            lockerProgram: LOCKER_PROGRAM_ID,
            lockerEventAuthority: deriveLockerEventAuthority(),
            systemProgram: SystemProgram.programId,
        })
        .preInstructions(preInstructions)
        .transaction();
    transaction.recentBlockhash = (await banksClient.getLatestBlockhash())[0];
    transaction.sign(payer);
    await processTransactionMaybeThrow(banksClient, transaction);
}

export const deriveLockerEscrow = (base: PublicKey) => {
    const [escrow] = PublicKey.findProgramAddressSync(
        [Buffer.from("escrow"), base.toBuffer()],
        LOCKER_PROGRAM_ID
    );
    return escrow;
}

export function deriveLockerEventAuthority() {
    return PublicKey.findProgramAddressSync(
        [Buffer.from("__event_authority")],
        LOCKER_PROGRAM_ID
    )[0];
}