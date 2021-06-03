import { Connection, PublicKey, Signer } from '@solana/web3.js'
import { CollateralLayout, LiquidityLayout, MarketLayout } from './layout'
import { Collateral, Liquidity, Market } from './state'

export const PROGRAM_ID: PublicKey = new PublicKey('69LK6qziCCnqgmUPYpuiJ2y8JavKVRrCZ4pDekSyDZTn')

export class LendingMarket {
  constructor(
    private connection: Connection,
    public pubkey: PublicKey,
    public programId: PublicKey,
    public payer: Signer,
  ) {}

  static init(connection: Connection, pubkey: PublicKey, payer: Signer) {
    return new LendingMarket(connection, pubkey, PROGRAM_ID, payer)
  }

  async getMarketInfo() {
    const info = await this.getOwnedAccountInfo(this.pubkey)
    if (info.data.length != MarketLayout.span) {
      throw new Error(`Invalid market size`)
    }

    const market = Market.from(info.data)
    return market
  }

  // TODO: replace to async iteration with cursors
  async getLiquidityTokens() {
    const market = await this.getMarketInfo()
    const [market_authority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    // TODO: replace to loop for BN
    const liquidityPubkeys = await Promise.all(
      [...Array(market.liquidityTokens.toNumber()).keys()].map((index: number) =>
        PublicKey.createWithSeed(market_authority, `liquidity${index}`, this.programId),
      ),
    )

    return Promise.all(liquidityPubkeys.map((pubkey: PublicKey) => this.getLiquidityInfo(pubkey)))
  }

  // TODO: replace to async iteration with cursors
  async getCollateralTokens() {
    const market = await this.getMarketInfo()
    const [market_authority] = await PublicKey.findProgramAddress(
      [this.pubkey.toBuffer()],
      this.programId,
    )

    // TODO: replace to loop for BN
    const collateralPubkeys = await Promise.all(
      [...Array(market.collateralTokens.toNumber()).keys()].map((index: number) =>
        PublicKey.createWithSeed(market_authority, `collateral${index}`, this.programId),
      ),
    )

    return Promise.all(collateralPubkeys.map((pubkey: PublicKey) => this.getCollateralInfo(pubkey)))
  }

  async getLiquidityInfo(liquidityPubkey: PublicKey) {
    const info = await this.getOwnedAccountInfo(liquidityPubkey)
    if (info.data.length != LiquidityLayout.span) {
      throw new Error(`Invalid liquidity size`)
    }

    const liquidity = Liquidity.from(info.data)
    return liquidity
  }

  async getCollateralInfo(collateralPubkey: PublicKey) {
    const info = await this.getOwnedAccountInfo(collateralPubkey)
    if (info.data.length != CollateralLayout.span) {
      throw new Error(`Invalid collateral size`)
    }

    const collateral = Collateral.from(info.data)
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
}
