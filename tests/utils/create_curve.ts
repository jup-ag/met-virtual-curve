import BN from "bn.js";
import { ConfigParameters, LiquidityDistributionParameters } from "../instructions";
import Decimal from "decimal.js";
import { MAX_SQRT_PRICE } from "./constants";



function getDeltaAmountBase(lowerSqrtPrice: BN, upperSqrtPrice: BN, liquidity: BN): BN {
    let numerator = liquidity.mul(upperSqrtPrice.sub(lowerSqrtPrice));
    let denominator = lowerSqrtPrice.mul(upperSqrtPrice);
    return numerator.add(denominator).sub(new BN(1)).div(denominator);
}
function getBaseTokenForSwap(
    sqrtStartPrice: BN,
    sqrtMigrationPrice: BN,
    curve: Array<LiquidityDistributionParameters>,
): BN {
    let totalAmount = new BN(0);
    for (let i = 0; i < curve.length; i++) {
        let lowerSqrtPrice = i == 0 ? sqrtStartPrice : curve[i - 1].sqrtPrice;
        if (curve[i].sqrtPrice > sqrtMigrationPrice) {
            let deltaAmount = getDeltaAmountBase(
                lowerSqrtPrice,
                sqrtMigrationPrice,
                curve[i].liquidity,
            );
            totalAmount = totalAmount.add(deltaAmount);
            break;
        } else {
            let deltaAmount = getDeltaAmountBase(
                lowerSqrtPrice,
                curve[i].sqrtPrice,
                curve[i].liquidity,
            );
            totalAmount = totalAmount.add(deltaAmount);
        }
    }
    return totalAmount;
}

function getBaseTokenForMigration(sqrtMigrationPrice: BN, migrationQuoteThreshold: BN): BN {
    let price = sqrtMigrationPrice.mul(sqrtMigrationPrice);
    let base = migrationQuoteThreshold.shln(128).div(price);
    return base;
}

// Original formula: price = (sqrtPrice >> 64)^2 * 10^(tokenADecimal - tokenBDecimal)
// Reverse formula: sqrtPrice = sqrt(price / 10^(tokenADecimal - tokenBDecimal)) << 64
export const getSqrtPriceFromPrice = (
    price: string,
    tokenADecimal: number,
    tokenBDecimal: number
): BN => {
    const decimalPrice = new Decimal(price);
    const adjustedByDecimals = decimalPrice.div(
        new Decimal(10 ** (tokenADecimal - tokenBDecimal))
    );
    const sqrtValue = Decimal.sqrt(adjustedByDecimals);
    const sqrtValueQ64 = sqrtValue.mul(Decimal.pow(2, 64));
    return new BN(sqrtValueQ64.floor().toFixed());
};

export const getPriceFromSqrtPrice = (
    sqrtPrice: BN,
    tokenADecimal: number,
    tokenBDecimal: number
): Decimal => {
    const decimalSqrtPrice = new Decimal(sqrtPrice.toString());
    const price = decimalSqrtPrice
        .mul(decimalSqrtPrice)
        .mul(new Decimal(10 ** (tokenADecimal - tokenBDecimal)))
        .div(Decimal.pow(2, 128));

    return price;
};

export function designCurve(
    totalTokenSupply: number,
    percentageSupplyOnMigration: number,
    percentageSupplyVesting: number,
    frequency: number,
    numberOfPeriod: number,
    startPrice: Decimal,
    migrationPrice: Decimal,
    tokenBaseDecimal: number,
    tokenQuoteDecimal: number,
): ConfigParameters {
    let totalSupply = new BN(totalTokenSupply).mul(new BN(10).pow(new BN(tokenBaseDecimal)));
    let baseDecimalFactor = new Decimal(10 ** tokenBaseDecimal);
    let quoteDecimalFactor = new Decimal(10 ** tokenQuoteDecimal);
    let preMigrationTokenSupply = totalSupply;
    let postMigrationTokenSupply = totalSupply;
    let migrationSupply = totalSupply.mul(new BN(percentageSupplyOnMigration)).div(new BN(100));
    let lockedVestingAmount = totalSupply.mul(new BN(percentageSupplyVesting)).div(new BN(100));
    let amountPerPeriod = numberOfPeriod == 0 ? new BN(0) : lockedVestingAmount.div(new BN(numberOfPeriod));
    lockedVestingAmount = amountPerPeriod.mul(new BN(numberOfPeriod));

    let sqrtStartPrice = getSqrtPriceFromPrice(startPrice, tokenBaseDecimal, tokenQuoteDecimal);
    let migrationSqrtPrice = getSqrtPriceFromPrice(migrationPrice, tokenBaseDecimal, tokenQuoteDecimal);
    let priceDelta = migrationSqrtPrice.sub(sqrtStartPrice);

    let migrationQuoteThresholdFloat = migrationPrice.mul(new Decimal(migrationSupply.toString())).mul(quoteDecimalFactor).div(baseDecimalFactor).floor();
    let migrationQuoteThreshold = new BN(migrationQuoteThresholdFloat.toString());
    let liquidity = migrationQuoteThreshold.shln(128).div(priceDelta);
    let curves = [
        {
            sqrtPrice: migrationSqrtPrice,
            liquidity,
        },
        {
            sqrtPrice: MAX_SQRT_PRICE,
            liquidity: new BN(1),
        }
    ];

    // reverse to get amount on swap
    let maxSwapAmount = getBaseTokenForSwap(sqrtStartPrice, MAX_SQRT_PRICE, curves);
    let migrationAmount = getBaseTokenForMigration(migrationSqrtPrice, migrationQuoteThreshold);
    let cliffUnlockAmount = percentageSupplyVesting == 0 ? new BN(0) : totalSupply.sub(maxSwapAmount).sub(lockedVestingAmount).sub(migrationAmount);

    console.log("migrationAmount: ", migrationAmount.toString());
    console.log("maxSwapAmount: ", maxSwapAmount.toString());
    console.log("migrationQuoteThreshold: ", migrationQuoteThreshold.toString());

    const instructionParams: ConfigParameters = {
        poolFees: {
            baseFee: {
                cliffFeeNumerator: new BN(2_500_000),
                numberOfPeriod: 0,
                reductionFactor: new BN(0),
                periodFrequency: new BN(0),
                feeSchedulerMode: 0,
            },
            dynamicFee: null,
        },
        activationType: 0,
        collectFeeMode: 1,
        migrationOption: 0, /// damm v1
        tokenType: 0, // spl_token
        tokenDecimal: tokenBaseDecimal,
        migrationQuoteThreshold,
        partnerLpPercentage: 0,
        creatorLpPercentage: 0,
        partnerLockedLpPercentage: 100,
        creatorLockedLpPercentage: 0,
        sqrtStartPrice,
        lockedVesting: {
            amountPerPeriod: amountPerPeriod,
            cliffDurationFromMigrationTime: new BN(0),
            frequency: new BN(frequency),
            numberOfPeriod: new BN(numberOfPeriod),
            cliffUnlockAmount,
        },
        migrationFeeOption: 0,
        tokenSupply: {
            preMigrationTokenSupply,
            postMigrationTokenSupply,
        },
        padding: [],
        curve: curves,
    };
    return instructionParams;
}



export function designCurveWihoutLockVesting(
    totalTokenSupply: number,
    percentageSupplyOnMigration: number,
    startPrice: Decimal,
    tokenBaseDecimal: number,
    tokenQuoteDecimal: number,
): ConfigParameters {
    let totalSupply = new BN(totalTokenSupply).mul(new BN(10).pow(new BN(tokenBaseDecimal)));
    let baseDecimalFactor = new Decimal(10 ** tokenBaseDecimal);
    let quoteDecimalFactor = new Decimal(10 ** tokenQuoteDecimal);
    let preMigrationTokenSupply = totalSupply;
    let postMigrationTokenSupply = totalSupply;
    let migrationSupply = totalSupply.mul(new BN(percentageSupplyOnMigration)).div(new BN(100));
    let swapSupply = totalSupply.sub(migrationSupply);

    let sqrtStartPrice = getSqrtPriceFromPrice(startPrice, tokenBaseDecimal, tokenQuoteDecimal);
    let migrationSqrtPrice = sqrtStartPrice.mul(swapSupply).div(migrationSupply); // magic formula
    migrationSqrtPrice = migrationSqrtPrice.sub(new BN(1));
    let priceDelta = migrationSqrtPrice.sub(sqrtStartPrice);

    let migrationPrice = getPriceFromSqrtPrice(migrationSqrtPrice, tokenBaseDecimal, tokenQuoteDecimal);
    let migrationQuoteThresholdFloat = migrationPrice.mul(new Decimal(migrationSupply.toString())).mul(quoteDecimalFactor).div(baseDecimalFactor).floor();

    let migrationQuoteThreshold = new BN(migrationQuoteThresholdFloat.toString());


    let liquidity = migrationQuoteThreshold.shln(128).div(priceDelta);
    let curves = [
        {
            sqrtPrice: migrationSqrtPrice,
            liquidity,
        },
        {
            sqrtPrice: MAX_SQRT_PRICE,
            liquidity: new BN(1),
        }
    ];

    const instructionParams: ConfigParameters = {
        poolFees: {
            baseFee: {
                cliffFeeNumerator: new BN(2_500_000),
                numberOfPeriod: 0,
                reductionFactor: new BN(0),
                periodFrequency: new BN(0),
                feeSchedulerMode: 0,
            },
            dynamicFee: null,
        },
        activationType: 0,
        collectFeeMode: 1,
        migrationOption: 0, /// damm v1
        tokenType: 0, // spl_token
        tokenDecimal: tokenBaseDecimal,
        migrationQuoteThreshold,
        partnerLpPercentage: 0,
        creatorLpPercentage: 0,
        partnerLockedLpPercentage: 100,
        creatorLockedLpPercentage: 0,
        sqrtStartPrice,
        lockedVesting: {
            amountPerPeriod: new BN(0),
            cliffDurationFromMigrationTime: new BN(0),
            frequency: new BN(0),
            numberOfPeriod: new BN(0),
            cliffUnlockAmount: new BN(0),
        },
        migrationFeeOption: 0,
        tokenSupply: {
            preMigrationTokenSupply,
            postMigrationTokenSupply,
        },
        padding: [],
        curve: curves,
    };
    return instructionParams;
}
