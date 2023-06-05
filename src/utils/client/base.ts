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


// const CLIENT_IDENTIFICATION: &str = "CLIENT_IDENTIFICATION";

// pub fn uniform_event_name(name: &str) -> String {
//     format!("{}::{}", CLIENT_IDENTIFICATION, name)
// }

const CLIENT_IDENTIFICATION = "CLIENT_IDENTIFICATION"

export const farmatEventName = (name: string) => {
    return `${CLIENT_IDENTIFICATION}::${name}`
}


