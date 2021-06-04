import { TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token'
import { PublicKey, TransactionInstruction } from '@solana/web3.js'
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
export const LiquidityDeposit = ({
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
