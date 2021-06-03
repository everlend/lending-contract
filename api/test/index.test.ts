import { LendingMarket } from '../src'
import { Connection, Keypair, PublicKey } from '@solana/web3.js'

const SECRET_KEY = Uint8Array.from([
  230, 130, 183, 211, 202, 141, 184, 115, 203, 212, 117, 219, 8, 19, 135, 200, 67, 52, 225, 10, 106,
  126, 118, 143, 20, 191, 14, 208, 157, 155, 199, 41, 109, 125, 225, 87, 230, 88, 40, 215, 184, 236,
  122, 125, 218, 233, 30, 111, 9, 20, 128, 200, 48, 109, 187, 135, 196, 140, 252, 2, 55, 207, 142,
  141,
])

const MARKET_PUBKEY: PublicKey = new PublicKey('3E1nNz4FiptsBW8vj36zQEikH4AYjhqSD3jfWkjc4LZV')
const LIQUIDITY_PUBKEY: PublicKey = new PublicKey('BrmEecfTGZFoygN4RVUvPC3wNeGoTx23sor8r9R12toX')
const COLLATERAL_PUBKEY: PublicKey = new PublicKey('A1EgEXQ4p3R6vgiv35gQNnc198QZ5D3YTL4edpmcnwQH')

describe('LendingMarket', () => {
  let lendingMarket: LendingMarket

  beforeAll(() => {
    const connection = new Connection('http://127.0.0.1:8899', 'recent')
    const payer = Keypair.fromSecretKey(SECRET_KEY)
    lendingMarket = LendingMarket.init(connection, MARKET_PUBKEY, payer)
  })

  describe('getMarketInfo', () => {
    test('get market info', async () => {
      const market = await lendingMarket.getMarketInfo()

      console.log(market)

      expect(market.version).toEqual(1)
    })
  })

  describe('getLiquidityInfo', () => {
    test('get liquidity info', async () => {
      const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)

      console.log(liquidity)

      expect(liquidity.market).toEqual(MARKET_PUBKEY)
    })
  })

  describe('getCollateralInfo', () => {
    test('get collateral info', async () => {
      const collateral = await lendingMarket.getCollateralInfo(COLLATERAL_PUBKEY)

      console.log(collateral)

      expect(collateral.market).toEqual(MARKET_PUBKEY)
    })
  })

  describe('getLiquidityTokens', () => {
    test('get liquidity tokens', async () => {
      const liquidityTokens = await lendingMarket.getLiquidityTokens()

      console.log(liquidityTokens)

      expect(liquidityTokens.length).toEqual(1)
    })
  })

  describe('getCollateralTokens', () => {
    test('get collateral tokens', async () => {
      const collateralTokens = await lendingMarket.getCollateralTokens()

      console.log(collateralTokens)

      expect(collateralTokens.length).toEqual(1)
    })
  })
})
