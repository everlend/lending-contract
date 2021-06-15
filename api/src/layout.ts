import BufferLayout from 'buffer-layout'
import * as BaseLayout from './baseLayout'

export const InstructionLayout = BufferLayout.u8('instruction')

export const MarketLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BaseLayout.publicKey('owner'),
  BaseLayout.uint64('liquidity_tokens'),
  BaseLayout.uint64('collateral_tokens'),
])

export const LiquidityLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BufferLayout.u8('status'),
  BaseLayout.publicKey('market'),
  BaseLayout.publicKey('token_mint'),
  BaseLayout.publicKey('token_account'),
  BaseLayout.publicKey('pool_mint'),
  BaseLayout.uint64('amount_borrowed'),
  BaseLayout.publicKey('oracle'),
])

export const CollateralLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BufferLayout.u8('status'),
  BaseLayout.publicKey('market'),
  BaseLayout.publicKey('token_mint'),
  BaseLayout.publicKey('token_account'),
  BaseLayout.uint64('ratio_initial'),
  BaseLayout.uint64('ratio_healthy'),
  BaseLayout.publicKey('oracle'),
])

export const ObligationLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BaseLayout.publicKey('market'),
  BaseLayout.publicKey('owner'),
  BaseLayout.publicKey('liquidity'),
  BaseLayout.publicKey('collateral'),
  BaseLayout.uint64('amount_liquidity_borrowed'),
  BaseLayout.uint64('amount_collateral_deposited'),
])

export const MarketInsructionLayouts = {
  LiquidityDeposit: {
    index: 5,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  LiquidityWithdraw: {
    index: 6,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  CreateObligation: {
    index: 7,
    layout: BufferLayout.struct([InstructionLayout]),
  },
  ObligationCollateralDeposit: {
    index: 8,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  ObligationCollateralWithdraw: {
    index: 9,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  ObligationLiquidityBorrow: {
    index: 10,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  ObligationLiquidityRepay: {
    index: 11,
    layout: BufferLayout.struct([InstructionLayout, BaseLayout.uint64('amount')]),
  },
  LiquidateObligation: {
    index: 12,
    layout: BufferLayout.struct([InstructionLayout]),
  },
}
