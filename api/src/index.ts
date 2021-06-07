import { MintLayout, Token, u64 } from '@solana/spl-token'
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  Transaction,
} from '@solana/web3.js'
import * as Instruction from './instruction'
import { CollateralLayout, LiquidityLayout, MarketLayout } from './layout'
import { Collateral, Liquidity, Market } from './state'
import { createTokenAccountTx } from './transaction'

export const PROGRAM_ID: PublicKey = new PublicKey('69LK6qziCCnqgmUPYpuiJ2y8JavKVRrCZ4pDekSyDZTn')

export * from './instruction'
export * from './transaction'
export * from './layout'
export * from './utils'

export class LendingMarket {
  private _payer: Signer

  constructor(
    private connection: Connection,
    public pubkey: PublicKey,
    public programId: PublicKey,
    payer?: Signer,
  ) {
    this._payer = payer
  }

  static init(connection: Connection, pubkey: PublicKey, payer?: Signer) {
    return new LendingMarket(connection, pubkey, PROGRAM_ID, payer)
  }

  get payer() {
    if (!this._payer) {
      throw new Error('Payer not specified')
    }
    return this._payer
  }

  async getMarketInfo() {
    const info = await this.getOwnedAccountInfo(this.pubkey)
    if (info.data.length != MarketLayout.span) {
      throw new Error(`Invalid market size`)
    }

    const market = { ...Market.from(info.data), pubkey: this.pubkey }
    return market
  }

  // TODO: replace to async iteration with cursors
  async getLiquidityTokens() {
    const market = await this.getMarketInfo()
    const [marketAuthority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    // TODO: replace to loop for BN
    const liquidityPubkeys = await Promise.all(
      [...Array(market.liquidityTokens.toNumber()).keys()].map((index: number) =>
        PublicKey.createWithSeed(marketAuthority, `liquidity${index}`, this.programId),
      ),
    )

    return Promise.all(liquidityPubkeys.map((pubkey: PublicKey) => this.getLiquidityInfo(pubkey)))
  }

  // TODO: replace to async iteration with cursors
  async getCollateralTokens() {
    const market = await this.getMarketInfo()
    const [marketAuthority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    // TODO: replace to loop for BN
    const collateralPubkeys = await Promise.all(
      [...Array(market.collateralTokens.toNumber()).keys()].map((index: number) =>
        PublicKey.createWithSeed(marketAuthority, `collateral${index}`, this.programId),
      ),
    )

    return Promise.all(collateralPubkeys.map((pubkey: PublicKey) => this.getCollateralInfo(pubkey)))
  }

  async getLiquidityInfo(liquidityPubkey: PublicKey) {
    const info = await this.getOwnedAccountInfo(liquidityPubkey)
    if (info.data.length != LiquidityLayout.span) {
      throw new Error(`Invalid liquidity size`)
    }

    const liquidity = { ...Liquidity.from(info.data), pubkey: liquidityPubkey }
    return liquidity
  }

  async getCollateralInfo(collateralPubkey: PublicKey) {
    const info = await this.getOwnedAccountInfo(collateralPubkey)
    if (info.data.length != CollateralLayout.span) {
      throw new Error(`Invalid collateral size`)
    }

    const collateral = { ...Collateral.from(info.data), pubkey: collateralPubkey }
    return collateral
  }

  async getOwnedAccountInfo(pubkey: PublicKey) {
    const info = await this.connection.getAccountInfo(pubkey)
    if (!info) {
      throw new Error('Failed to find account')
    }

    if (!info.owner.equals(this.programId)) {
      throw new Error(`Invalid owner: ${JSON.stringify(info.owner)}`)
    }

    return info
  }

  /**
   * Generate liquidity accounts for source & destination
   * @param liquidityPubkey Liquidity pubkey
   * @returns [token account, pool account]
   */
  async generateLiquidityAccounts(liquidityPubkey: PublicKey) {
    const liquidity = await this.getLiquidityInfo(liquidityPubkey)
    const rent = await Token.getMinBalanceRentForExemptAccount(this.connection)
    const tokenAccount = Keypair.generate()
    const poolAccount = Keypair.generate()

    const tx = new Transaction().add(
      createTokenAccountTx(
        this.payer.publicKey,
        tokenAccount.publicKey,
        liquidity.tokenMint,
        this.payer.publicKey,
        rent,
      ),
      createTokenAccountTx(
        this.payer.publicKey,
        poolAccount.publicKey,
        liquidity.poolMint,
        this.payer.publicKey,
        rent,
      ),
    )
    const signature = await sendAndConfirmTransaction(this.connection, tx, [
      this.payer,
      tokenAccount,
      poolAccount,
    ])
    console.log(`Signature: ${signature}`)

    return [tokenAccount.publicKey, poolAccount.publicKey]
  }

  /**
   * Generate collateral source account
   * @param collateralPubkey Collateral Pubkey
   * @returns [token account]
   */
  async generateCollateralAccounts(collateralPubkey: PublicKey) {
    const collateral = await this.getCollateralInfo(collateralPubkey)
    const rent = await Token.getMinBalanceRentForExemptAccount(this.connection)
    const tokenAccount = Keypair.generate()

    const tx = createTokenAccountTx(
      this.payer.publicKey,
      tokenAccount.publicKey,
      collateral.tokenMint,
      this.payer.publicKey,
      rent,
    )

    const signature = await sendAndConfirmTransaction(this.connection, tx, [
      this.payer,
      tokenAccount,
    ])
    console.log(`Signature: ${signature}`)

    return [tokenAccount.publicKey]
  }

  async liquidityDepositTx(
    liquidityPubkey: PublicKey,
    uiAmount: number,
    source: PublicKey,
    destination: PublicKey,
    userTransferAuthority: PublicKey,
  ) {
    const liquidity = await this.getLiquidityInfo(liquidityPubkey)

    const [marketAuthority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    const amount = new u64(uiAmount * Math.pow(10, await this.getMintDecimals(liquidity.tokenMint)))

    const tx = new Transaction().add(
      Instruction.liquidityDeposit({
        programId: this.programId,
        market: this.pubkey,
        liquidity: liquidityPubkey,
        source,
        destination,
        tokenAccount: liquidity.tokenAccount,
        poolMint: liquidity.poolMint,
        marketAuthority,
        userTransferAuthority,
        amount,
      }),
    )

    return tx
  }

  /**
   * Transfer tokens to liquidity account and mint pool tokens
   * @param liquidityPubkey Liquidity pubkey
   * @param uiAmount Amount tokens to deposit
   * @param source Source account of token mint
   * @param destination Destination account of pool mint
   * @param payer Signer for transfer tokens
   */
  async liquidityDeposit(
    liquidityPubkey: PublicKey,
    uiAmount: number,
    source: PublicKey,
    destination: PublicKey,
  ) {
    const tx = await this.liquidityDepositTx(
      liquidityPubkey,
      uiAmount,
      source,
      destination,
      this.payer.publicKey,
    )

    const signature = await sendAndConfirmTransaction(this.connection, tx, [this.payer])
    console.log(`Signature: ${signature}`)
  }

  async liquidityWithdrawTx(
    liquidityPubkey: PublicKey,
    uiAmount: number,
    source: PublicKey,
    destination: PublicKey,
    userTransferAuthority: PublicKey,
  ) {
    const liquidity = await this.getLiquidityInfo(liquidityPubkey)

    const [marketAuthority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    const amount = new u64(uiAmount * Math.pow(10, await this.getMintDecimals(liquidity.poolMint)))

    const tx = new Transaction().add(
      Instruction.liquidityWithdraw({
        programId: this.programId,
        market: this.pubkey,
        liquidity: liquidityPubkey,
        source,
        destination,
        tokenAccount: liquidity.tokenAccount,
        poolMint: liquidity.poolMint,
        marketAuthority,
        userTransferAuthority,
        amount,
      }),
    )

    return tx
  }

  /**
   * Burn pool tokens and transfer liquidity tokens
   * @param liquidityPubkey Liquidity pubkey
   * @param uiAmount Amount tokens to deposit
   * @param source Source account of pool mint
   * @param destination Destination account of token account
   * @param payer Signer for transfer tokens
   */
  async liquidityWithdraw(
    liquidityPubkey: PublicKey,
    uiAmount: number,
    source: PublicKey,
    destination: PublicKey,
  ) {
    const tx = await this.liquidityWithdrawTx(
      liquidityPubkey,
      uiAmount,
      source,
      destination,
      this.payer.publicKey,
    )

    const signature = await sendAndConfirmTransaction(this.connection, tx, [this.payer])
    console.log(`Signature: ${signature}`)
  }

  async createObligationTx(
    liquidityPubkey: PublicKey,
    collateralPubkey: PublicKey,
    owner: PublicKey,
  ): Promise<{ tx: Transaction; pubkey: PublicKey }> {
    const [obligationAuthority] = await PublicKey.findProgramAddress(
      [
        owner.toBuffer(),
        this.pubkey.toBuffer(),
        liquidityPubkey.toBuffer(),
        collateralPubkey.toBuffer(),
      ],
      this.programId,
    )

    const obligationPubkey = await PublicKey.createWithSeed(
      obligationAuthority,
      'obligation',
      this.programId,
    )

    const tx = new Transaction().add(
      Instruction.createObligation({
        programId: this.programId,
        market: this.pubkey,
        obligation: obligationPubkey,
        liquidity: liquidityPubkey,
        collateral: collateralPubkey,
        obligationAuthority,
        owner,
      }),
    )

    return { tx, pubkey: obligationPubkey }
  }

  async createObligation(
    liquidityPubkey: PublicKey,
    collateralPubkey: PublicKey,
  ): Promise<PublicKey> {
    const { tx, pubkey } = await this.createObligationTx(
      liquidityPubkey,
      collateralPubkey,
      this.payer.publicKey,
    )

    const signature = await sendAndConfirmTransaction(this.connection, tx, [this.payer])
    console.log(`Signature: ${signature}`)

    return pubkey
  }

  async getMintDecimals(pubkey: PublicKey) {
    const mintInfo = await this.connection.getAccountInfo(pubkey)
    const mint = MintLayout.decode(mintInfo.data)

    return mint.decimals
  }
}
