export default {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['**/*.(spec|test).ts', '**/test/*.ts'],
  globals: {
    'ts-jest': {
      tsconfig: './tsconfig.json',
      diagnostics: false,
    },
  },
}
