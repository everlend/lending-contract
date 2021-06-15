import { Token, TOKEN_PROGRAM_ID, u64 } from '@solana/spl-token'
import { Connection, Keypair, PublicKey } from '@solana/web3.js'
import { LendingMarket } from '../src'

const SECRET_KEY = Uint8Array.from([
  230, 130, 183, 211, 202, 141, 184, 115, 203, 212, 117, 219, 8, 19, 135, 200, 67, 52, 225, 10, 106,
  126, 118, 143, 20, 191, 14, 208, 157, 155, 199, 41, 109, 125, 225, 87, 230, 88, 40, 215, 184, 236,
  122, 125, 218, 233, 30, 111, 9, 20, 128, 200, 48, 109, 187, 135, 196, 140, 252, 2, 55, 207, 142,
  141,
])

const ENDPOINT = 'https://api.devnet.solana.com'
const MARKET_PUBKEY: PublicKey = new PublicKey('31dcWJrN1a2QtS2gULCzXZwtA61ej6R5dnyw3jxfRrun')
const LIQUIDITY_PUBKEY: PublicKey = new PublicKey('829Jka8s5qdeJzd16PUQZDpFTRZzDx6VVKzHUsxzfqqn')
const COLLATERAL_PUBKEY: PublicKey = new PublicKey('4vAS1K5D6RoTPFrszfkbu3o9pCDj8gL1LdHBdyUdEqPt')
const RATIO_INITIAL = 0.5

describe('LendingMarket', () => {
  let payer: Keypair
  let connection: Connection
  let lendingMarket: LendingMarket

  const generateNewPayer = async () => {
    const newPayer = Keypair.generate()
    const airdropTx = await connection.requestAirdrop(newPayer.publicKey, 1000000000)
    await connection.confirmTransaction(airdropTx)

    return newPayer
  }

  const prepareProvider = async (provider: Keypair) => {
    const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)
    const tokenMint = new Token(connection, liquidity.tokenMint, TOKEN_PROGRAM_ID, payer)

    const [liquidityTokenAccount, poolAccount] = await lendingMarket.generateProviderAccounts(
      LIQUIDITY_PUBKEY,
      provider,
    )

    await tokenMint.mintTo(liquidityTokenAccount, payer, [], 999999999999)

    return [liquidityTokenAccount, poolAccount]
  }

  const prepareBorrower = async (borrower: Keypair) => {
    const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)
    const collateral = await lendingMarket.getCollateralInfo(COLLATERAL_PUBKEY)

    const [liquidityTokenAccount, collateralTokenAccount] =
      await lendingMarket.generateBorrowerAccounts(LIQUIDITY_PUBKEY, COLLATERAL_PUBKEY, borrower)

    const liquidityTokenMint = new Token(connection, liquidity.tokenMint, TOKEN_PROGRAM_ID, payer)
    const collateralTokenMint = new Token(connection, collateral.tokenMint, TOKEN_PROGRAM_ID, payer)

    await liquidityTokenMint.mintTo(liquidityTokenAccount, payer, [], 999999999999)
    await collateralTokenMint.mintTo(collateralTokenAccount, payer, [], 999999999999)

    return [liquidityTokenAccount, collateralTokenAccount]
  }

  beforeAll(() => {
    connection = new Connection(ENDPOINT, 'recent')
    payer = Keypair.fromSecretKey(SECRET_KEY)
    lendingMarket = LendingMarket.init(connection, MARKET_PUBKEY, payer)
  })

  describe('getMarketInfo', () => {
    test('get market info', async () => {
      const market = await lendingMarket.getMarketInfo()

      expect(market.version).toEqual(1)
    })
  })

  describe('getLiquidityInfo', () => {
    test('get liquidity info', async () => {
      const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)

      expect(liquidity.market).toEqual(MARKET_PUBKEY)
    })
  })

  describe('getCollateralInfo', () => {
    test('get collateral info', async () => {
      const collateral = await lendingMarket.getCollateralInfo(COLLATERAL_PUBKEY)

      expect(collateral.market).toEqual(MARKET_PUBKEY)
    })
  })

  describe('getLiquidityTokens', () => {
    test('get liquidity tokens', async () => {
      const liquidityTokens = await lendingMarket.getLiquidityTokens()

      expect(liquidityTokens.length).toEqual(1)
    })
  })

  describe('getCollateralTokens', () => {
    test('get collateral tokens', async () => {
      const collateralTokens = await lendingMarket.getCollateralTokens()

      expect(collateralTokens.length).toEqual(1)
    })
  })

  describe.skip('liquidityDeposit', () => {
    test('liquidity deposit', async () => {
      const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)
      const tokenMint = new Token(connection, liquidity.tokenMint, TOKEN_PROGRAM_ID, payer)
      const [source, destination] = await prepareProvider(payer)

      const uiAmount = 0.05
      const amount = new u64(
        uiAmount * Math.pow(10, await lendingMarket.getMintDecimals(liquidity.tokenMint)),
      )

      const balanceBefore = (await tokenMint.getAccountInfo(liquidity.tokenAccount)).amount
      await lendingMarket.liquidityDeposit(LIQUIDITY_PUBKEY, uiAmount, source, destination)

      const balanceAfter = (await tokenMint.getAccountInfo(liquidity.tokenAccount)).amount
      expect(balanceAfter.cmp(balanceBefore.add(amount))).toEqual(0)
    })
  })

  describe.skip('liquidityWithdraw', () => {
    test('liquidity withdraw', async () => {
      const liquidity = await lendingMarket.getLiquidityInfo(LIQUIDITY_PUBKEY)
      const tokenMint = new Token(connection, liquidity.tokenMint, TOKEN_PROGRAM_ID, payer)
      const [source, destination] = await prepareProvider(payer)

      const uiAmount = 0.05
      const amount = new u64(
        uiAmount * Math.pow(10, await lendingMarket.getMintDecimals(liquidity.tokenMint)),
      )

      await tokenMint.mintTo(source, payer, [], 999999999999)
      await lendingMarket.liquidityDeposit(LIQUIDITY_PUBKEY, uiAmount, source, destination)

      const balanceBefore = (await tokenMint.getAccountInfo(liquidity.tokenAccount)).amount
      await lendingMarket.liquidityWithdraw(LIQUIDITY_PUBKEY, uiAmount, destination, source)

      const balanceAfter = (await tokenMint.getAccountInfo(liquidity.tokenAccount)).amount
      expect(balanceAfter.cmp(balanceBefore.sub(amount))).toEqual(0)
    })
  })

  describe.skip('createObligation', () => {
    test('create obligation', async () => {
      const borrower = await generateNewPayer()
      const obligationPubkey = await lendingMarket.createObligation(
        LIQUIDITY_PUBKEY,
        COLLATERAL_PUBKEY,
        borrower,
      )

      const obligation = await lendingMarket.getObligationInfo(obligationPubkey)
      expect(obligation.owner.toString()).toEqual(borrower.publicKey.toString())
    })
  })

  describe.skip('obligationCollateralDeposit', () => {
    test('obligation collateral deposit', async () => {
      const borrower = await generateNewPayer()
      const obligationPubkey = await lendingMarket.createObligation(
        LIQUIDITY_PUBKEY,
        COLLATERAL_PUBKEY,
        borrower,
      )
      const [, source] = await prepareBorrower(borrower)
      const uiAmount = 0.05

      await lendingMarket.obligationCollateralDeposit(obligationPubkey, uiAmount, source, borrower)
    })
  })

  describe.skip('obligationCollateralWithdraw', () => {
    test('obligation collateral withdraw', async () => {
      const borrower = await generateNewPayer()
      const obligationPubkey = await lendingMarket.createObligation(
        LIQUIDITY_PUBKEY,
        COLLATERAL_PUBKEY,
        borrower,
      )
      const [, source] = await prepareBorrower(borrower)
      const uiAmount = 0.05

      await lendingMarket.obligationCollateralDeposit(obligationPubkey, uiAmount, source, borrower)
      await lendingMarket.obligationCollateralWithdraw(obligationPubkey, uiAmount, source, borrower)
    })
  })

  describe.skip('obligationLiquidityBorrow', () => {
    test('obligation liquidity borrow', async () => {
      // Liquidity deposit
      const [providerSource, providerDestination] = await prepareProvider(payer)
      await lendingMarket.liquidityDeposit(
        LIQUIDITY_PUBKEY,
        0.1,
        providerSource,
        providerDestination,
      )

      const borrower = await generateNewPayer()
      const obligationPubkey = await lendingMarket.createObligation(
        LIQUIDITY_PUBKEY,
        COLLATERAL_PUBKEY,
        borrower,
      )
      const [destination, source] = await prepareBorrower(borrower)
      const uiAmount = 0.05

      await lendingMarket.obligationCollateralDeposit(obligationPubkey, uiAmount, source, borrower)

      await lendingMarket.obligationLiquidityBorrow(
        obligationPubkey,
        uiAmount * RATIO_INITIAL,
        destination,
        borrower,
      )
    }, 10000)
  })

  describe.skip('obligationLiquidityRepay', () => {
    test('obligation liquidity repay', async () => {
      // Liquidity deposit
      const [providerSource, providerDestination] = await prepareProvider(payer)
      await lendingMarket.liquidityDeposit(
        LIQUIDITY_PUBKEY,
        0.1,
        providerSource,
        providerDestination,
      )

      const borrower = await generateNewPayer()
      const obligationPubkey = await lendingMarket.createObligation(
        LIQUIDITY_PUBKEY,
        COLLATERAL_PUBKEY,
        borrower,
      )
      const [destination, source] = await prepareBorrower(borrower)
      const uiAmount = 0.05

      await lendingMarket.obligationCollateralDeposit(obligationPubkey, uiAmount, source, borrower)

      await lendingMarket.obligationLiquidityBorrow(
        obligationPubkey,
        uiAmount * RATIO_INITIAL,
        destination,
        borrower,
      )

      await lendingMarket.obligationLiquidityRepay(
        obligationPubkey,
        uiAmount * RATIO_INITIAL,
        destination,
        borrower,
      )
    }, 10000)
  })
})
