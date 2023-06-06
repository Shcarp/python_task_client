/**
 * @file websocket client
 * @description websocket client
 * @module utils/client/websocket
 * @packageDocumentation
 * 随便写写, 有空再整理
 * @internal
 */

import EventEmitter from "events";
import { invoke } from "@tauri-apps/api";
import { Client, LocalResponse, MessageType, farmatEventName } from "./base";
import { appWindow } from "@tauri-apps/api/window";
import { Event } from "@tauri-apps/api/event";
import { Events } from "..";

enum PushStatus {
    SUCCESS = 0,
    ERROR = 1,
    WARN = 2,
}

type EeventPayload<T> = {
    event: string
    payload: T
    status: PushStatus
}

export enum State {
    INIT,
    CONNECTING,
    CONNECTED,
    RECONNECTING,
    CLOSEING,
    CLOSED,
}

type WebsocketEvent = {
    ['block_num']: (data: number) => void;
    ['connect']: () => void;
}

export class WebsocketClient extends EventEmitter implements Client {
    
    on: Events<WebsocketEvent>["on"] = super.on;
    emit: Events<WebsocketEvent>["emit"] = super.emit;

    client_id: string | undefined
    state: State = State.INIT

    constructor() {
        super()
        // 监听插件消息
    }
    
    private start() {
        if (!this.client_id && this.state !== State.CONNECTED) {
            return
        }

        appWindow.listen(farmatEventName("send_success"), (message) => {
            console.log("send_success", message)
        })

        appWindow.listen(farmatEventName("push"), (message: Event<EeventPayload<any>>) => {
            console.log("push", message)
            this.emit(message.payload?.event, message.payload)
        })
    }

    async connect(address = "ws://127.0.0.1:9673") {
        if(this.client_id) {
            return
        }
        const res: LocalResponse<string> = await invoke('plugin:connect|connect', {
            address: address
        })

        if (res.code === 0) {
            this.client_id = res.data
            this.state = State.CONNECTED
            this.start()
        }

        this.emit("connect")

    }

    async disconnect() {
        if (this.client_id) {
            invoke('plugin:connect|disconnect', {
                client_id: this.client_id
            })
        }
    }

    send<T>(url: string, data: MessageType): Promise<T> {
        if (this.state !== State.CONNECTED) {
            return Promise.reject("Client not connected")
        }
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
