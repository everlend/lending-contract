import { TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token'
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js'
import { MarketInsructionLayouts } from './layout'
import { encodeData } from './utils'

type BaseInstructionParams = { programId: PublicKey }

export type LiquidityDepositParams = BaseInstructionParams & {
  market: PublicKey
  liquidity: PublicKey
  source: PublicKey
  destination: PublicKey
  tokenAccount: PublicKey
  poolMint: PublicKey
  marketAuthority: PublicKey
  userTransferAuthority: PublicKey
  amount: u64
}
export const liquidityDeposit = ({
  programId,
  market,
  liquidity,
  source,
  destination,
  tokenAccount,
  poolMint,
  marketAuthority,
  userTransferAuthority,
  amount,
}: LiquidityDepositParams) => {
  const data = encodeData(MarketInsructionLayouts.LiquidityDeposit, {
    amount: new u64(amount).toBuffer(),
  })

  return new TransactionInstruction({
    keys: [
      { pubkey: liquidity, isSigner: false, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
      { pubkey: tokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolMint, isSigner: false, isWritable: true },
      { pubkey: market, isSigner: false, isWritable: false },
      { pubkey: marketAuthority, isSigner: false, isWritable: false },
      { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: new PublicKey(programId),
    data,
  })
}

export type LiquidityWithdrawParams = BaseInstructionParams & {
  market: PublicKey
  liquidity: PublicKey
  source: PublicKey
  destination: PublicKey
  tokenAccount: PublicKey
  poolMint: PublicKey
  marketAuthority: PublicKey
  userTransferAuthority: PublicKey
  amount: u64
}
export const liquidityWithdraw = ({
  programId,
  market,
  liquidity,
  source,
  destination,
  tokenAccount,
  poolMint,
  marketAuthority,
  userTransferAuthority,
  amount,
}: LiquidityWithdrawParams) => {
  const data = encodeData(MarketInsructionLayouts.LiquidityWithdraw, {
    amount: new u64(amount).toBuffer(),
  })

  return new TransactionInstruction({
    keys: [
      { pubkey: liquidity, isSigner: false, isWritable: false },
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
      { pubkey: tokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolMint, isSigner: false, isWritable: true },
      { pubkey: market, isSigner: false, isWritable: false },
      { pubkey: marketAuthority, isSigner: false, isWritable: false },
      { pubkey: userTransferAuthority, isSigner: true, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: new PublicKey(programId),
    data,
  })
}

export type CreateObligationParams = BaseInstructionParams & {
  market: PublicKey
  obligation: PublicKey
  liquidity: PublicKey
  collateral: PublicKey
  obligationAuthority: PublicKey
  owner: PublicKey
}
export const createObligation = ({
  programId,
  market,
  obligation,
  liquidity,
  collateral,
  obligationAuthority,
  owner,
}: CreateObligationParams) => {
  const data = encodeData(MarketInsructionLayouts.CreateObligation)

  return new TransactionInstruction({
    keys: [
      { pubkey: obligation, isSigner: false, isWritable: true },
      { pubkey: liquidity, isSigner: false, isWritable: false },
      { pubkey: collateral, isSigner: false, isWritable: false },
      { pubkey: market, isSigner: false, isWritable: false },
      { pubkey: obligationAuthority, isSigner: false, isWritable: false },
      { pubkey: owner, isSigner: true, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: new PublicKey(programId),
    data,
  })
}
