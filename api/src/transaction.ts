import { AccountLayout, Token, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js'

// const rent = await Token.getMinBalanceRentForExemptAccount(this.connection)

export const createTokenAccountTx = (
  from: PublicKey,
  account: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  rent: number,
) => {
  const tx = new Transaction()
  tx.add(
    SystemProgram.createAccount({
      fromPubkey: from,
      newAccountPubkey: account,
      lamports: rent,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
    }),
  )

  tx.add(Token.createInitAccountInstruction(TOKEN_PROGRAM_ID, mint, account, owner))

  return tx
}
