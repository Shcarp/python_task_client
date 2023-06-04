
export interface Server {
    connect: (address: string) => void
    disconnect: () => void
    send: (message: any) => void
}
