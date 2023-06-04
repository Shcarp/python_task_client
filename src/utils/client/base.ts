export type MessageType = number | string | boolean | object | null | undefined | Array<MessageType>

export interface LocalResponse<T> {
    code: number,
    data: T
}

export interface Client {
    connect: (address: string) => void
    disconnect: () => void
    send: (url: string, message: MessageType) => void
}
