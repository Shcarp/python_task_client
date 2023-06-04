import { invoke } from "@tauri-apps/api";
import { Client, LocalResponse, MessageType } from "./base";

export class WebsocketClient implements Client {
    
    client_id: string | undefined

    async connect(address = "ws://127.0.0.1:9673") {
        const res: LocalResponse<string> = await invoke('plugin:connect|connect', {
            address: address
        })

        if (res.code === 0) {
            this.client_id = JSON.parse(res.data)
        }
    }

    async disconnect() {
        if (this.client_id) {
            invoke('plugin:connect|disconnect', {
                client_id: this.client_id
            })
        }
    }

    send<T>(url: string, data: MessageType): Promise<T> {
        return new Promise<T>(async (resolve, reject) => {
            if (this.client_id) {
                const res: LocalResponse<T> = await invoke('plugin:connect|send', {
                    id: this.client_id,
                    url: url,
                    data
                })
                if (res.code === 0) {
                    resolve(res.data)
                }
                reject(res.data)
            }
           reject("Client not connected")
        })
    }
}
