import { u64 } from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'
import { CollateralLayout, LiquidityLayout, MarketLayout } from './layout'

export class Market {
  constructor(
    public version: number,
    public owner: PublicKey,
    public liquidityTokens: u64,
    public collateralTokens: u64,
  ) {}

  static from(buffer: Buffer): Market {
    const { version, owner, liquidity_tokens, collateral_tokens } = MarketLayout.decode(buffer)

    return {
      version,
      owner: new PublicKey(owner),
      liquidityTokens: u64.fromBuffer(liquidity_tokens),
      collateralTokens: u64.fromBuffer(collateral_tokens),
    }
  }
}

export enum LiquidityStatus {
  InActive = 0,
  Active = 1,
  InActiveAndVisible = 2,
}

export class Liquidity {
  constructor(
    public version: number,
    public status: LiquidityStatus,
    public market: PublicKey,
    public tokenMint: PublicKey,
    public tokenAccount: PublicKey,
    public poolMint: PublicKey,
  ) {}

  static from(buffer: Buffer): Liquidity {
    const { version, status, market, token_mint, token_account, pool_mint } =
      LiquidityLayout.decode(buffer)

    return {
      version,
      status: status as LiquidityStatus,
      market: new PublicKey(market),
      tokenMint: new PublicKey(token_mint),
      tokenAccount: new PublicKey(token_account),
      poolMint: new PublicKey(pool_mint),
    }
  }
}

export enum CollateralStatus {
  InActive = 0,
  Active = 1,
  InActiveAndVisible = 2,
}

export class Collateral {
  constructor(
    public version: number,
    public status: CollateralStatus,
    public market: PublicKey,
    public tokenMint: PublicKey,
    public tokenAccount: PublicKey,
    public ratioInitial: u64,
    public ratioHealthy: u64,
  ) {}

  static from(buffer: Buffer): Collateral {
    const { version, status, market, token_mint, token_account, ratio_initial, ratio_healthy } =
      CollateralLayout.decode(buffer)

    return {
      version,
      status: status as CollateralStatus,
      market: new PublicKey(market),
      tokenMint: new PublicKey(token_mint),
      tokenAccount: new PublicKey(token_account),
      ratioInitial: u64.fromBuffer(ratio_initial),
      ratioHealthy: u64.fromBuffer(ratio_healthy),
    }
  }
}
