import { ProgramTestContext } from "solana-bankrun";
import {
    createConfigMetadata,
} from "./instructions";
import { Pool, VirtualCurveProgram } from "./utils/types";
import { Keypair, PublicKey } from "@solana/web3.js";
import { fundSol, startTest } from "./utils";
import {
    createVirtualCurveProgram,
} from "./utils";

describe("Create config metadata", () => {
    let context: ProgramTestContext;
    let partner: Keypair;
    let user: Keypair;
    let program: VirtualCurveProgram;

    before(async () => {
        context = await startTest();
        user = context.payer;
        partner = Keypair.generate();
        program = createVirtualCurveProgram();
    });

    it("Partner create a config metadata", async () => {
        await createConfigMetadata(
            context.banksClient,
            program,
            {
                name: "Moonshot",
                website: "moonshot.com",
                logo: "https://raw.githubusercontent.com/MeteoraAg/token-metadata/main/meteora_permission_lp.png",
                feeClaimer: partner,
                payer: user,
            }
        );
    });
});
