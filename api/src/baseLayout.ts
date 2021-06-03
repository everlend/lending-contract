import BufferLayout from 'buffer-layout'

export const publicKey = (property = 'publicKey'): Record<string, unknown> => {
  return BufferLayout.blob(32, property)
}

export const uint64 = (property = 'uint64'): Record<string, unknown> => {
  return BufferLayout.blob(8, property)
}
