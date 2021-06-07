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
  BufferLayout.uint64('amount_borrowed'),
])

export const CollateralLayout = BufferLayout.struct([
  BufferLayout.u8('version'),
  BufferLayout.u8('status'),
  BaseLayout.publicKey('market'),
  BaseLayout.publicKey('token_mint'),
  BaseLayout.publicKey('token_account'),
  BaseLayout.uint64('ratio_initial'),
  BaseLayout.uint64('ratio_healthy'),
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
}
