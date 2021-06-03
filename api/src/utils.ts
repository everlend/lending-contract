import BufferLayout from 'buffer-layout'

export type InstructionType = {
  index: number
  layout: typeof BufferLayout
}

export const encodeData = (type: InstructionType, fields: Record<string, unknown> = {}): Buffer => {
  const data = Buffer.alloc(type.layout.span)
  type.layout.encode({ instruction: type.index, ...fields }, data)
  return data
}
