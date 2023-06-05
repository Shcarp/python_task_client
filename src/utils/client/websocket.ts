import { invoke } from "@tauri-apps/api";
import { Client, LocalResponse, MessageType, farmatEventName } from "./base";
import { appWindow } from "@tauri-apps/api/window";
// import EventEmitter from "events";

export class WebsocketClient implements Client {

    constructor() {
        // super()
        // 监听插件消息

        // appWindow
        appWindow.listen(farmatEventName("send_success"), (message) => {
            console.log("send_success", message)
        })
    }
    
    client_id: string | undefined

    async connect(address = "ws://127.0.0.1:9673") {
        if(this.client_id) {
            return
        }
        const res: LocalResponse<string> = await invoke('plugin:connect|connect', {
            address: address
        })

        if (res.code === 0) {
            this.client_id = res.data
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
