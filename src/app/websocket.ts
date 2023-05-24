// export const ws = new WebSocket("ws://localhost:9673";

import { message } from "antd";
// import { WebSocket } from "ws";
import { generate_messageid } from "./utils";
import EventEmitter from "events";
import { info, error as logError, warn, debug } from "tauri-plugin-log-api";

enum TimeUnit {
    Second = 1000,
    Millisecond = 1,
}

export let sleep = async (second: number, Unit: TimeUnit = TimeUnit.Second): Promise<void> => {
    return new Promise((resolve, reject) => {
        setTimeout(() => {
            resolve();
        }, second * Unit);
    });
};

export enum State {
    INIT,
    CONNECTING,
    CONNECTED,
    RECONNECTING,
    CLOSEING,
    CLOSED,
}

export enum MessageType {
    PING = "ping",
    PONG = "pong",
    RESPONSE = "response",
    PUSH = "push",
}

interface MessageData {
    type: MessageType;
    sequence: string;
    status: number;
    data: any;
}

export interface Message<T> {
    event: string;
    data: T;
}

const heartbeatInterval = 1000 * 60 * 2; // 2分钟

// 请求
export class Request<T> {
    sequence: string;
    sendTime: number;
    data: T;
    callback: (response: Response<any>) => void;
    constructor(sequence: string, data: T, callback: (response: Response<any>) => void) {
        this.sequence = sequence;
        this.sendTime = Date.now();
        this.data = data;
        this.callback = callback;
    }
}

export class Response<T> {
    sequence: string;
    status: number;
    data: T;
    constructor(sequence: string, data: MessageData) {
        this.sequence = sequence;
        this.status = data.status;
        this.data = data.data;
    }
}

export class WebSocketConnect extends EventEmitter {
    private url: string;
    private state: State = State.INIT;
    private _ws: WebSocket | undefined;
    // 全双工请求队列， 用来保存请求
    private sendq = new Map<string, Request<any>>();

    constructor(ip: string, port: number) {
        super();
        this.url = "ws://" + ip + ":" + port;
        this.doConnect();
    }

    get ws() {
        return this._ws;
    }

    set ws(ws: WebSocket | undefined) {
        this._ws = ws;
    }

    private async doConnect() {
        if (this.state != State.INIT) {
            return;
        }
        try {
            await this.connect();
        } catch (error) {
            info("websocket connect error");
            this.errorHandler(error as Error);
        }
    }

    private connect() {
        return new Promise<void>((resolve, reject) => {
            this.ws = new WebSocket(this.url);
            this.state = State.CONNECTING;
            this.ws.onopen = () => {
                this.state = State.CONNECTED;
                info("websocket connected");
                resolve();
            };
            this.ws.onclose = (ev) => {
                if (this.state === State.CLOSEING) {
                    info("websocket closed");
                    return;
                }
                if (this.state === State.INIT || this.state === State.RECONNECTING || this.state === State.CONNECTING) {
                    reject("init websocket close");
                    return;
                }
                debug("event[onclose] fired");
                this.errorHandler(new Error("server close"));
            };
            this.ws.onerror = (error) => {
                console.log(error);
                if (this.state === State.INIT || this.state === State.RECONNECTING || this.state === State.CONNECTING) {
                    reject("init websocket close");
                    return;
                }
                logError("websocket error");
                this.errorHandler(error as unknown as Error);
            };
            this.ws.onmessage = (event: MessageEvent) => {
                this.handleMessage(event);
            };
        });
    }

    protected handleMessage(event: MessageEvent) {
        const data = JSON.parse(event.data) as MessageData;
        if (data.status > 400) {
            this.ws?.close();
            info(`>>> error message ${data}`);
            return;
        }
        switch (data.type) {
            case MessageType.PING:
                this.send("pong");
                break;
            case MessageType.PONG:
                break;
            case MessageType.RESPONSE:
                const response = new Response(data.sequence, data);
                const request = this.sendq.get(response.sequence);
                if (request) {
                    request.callback(response);
                } else {
                    info(`>>> request not found ${response.sequence}`);
                }
                break;
            case MessageType.PUSH:
                this.emit("push", data.data);
                break;
            default:
                info(`>>> unknown message type ${data.type}`);
                break;
        }
    }

    protected request<T, S>(data: T): Promise<S> {
        return new Promise(async (resolve, reject) => {
            if (this.state !== State.CONNECTED) {
                reject("websocket not connected");
                return;
            }
            const sequence = await generate_messageid();
            const tr = setTimeout(() => {
                const request = this.sendq.get(sequence);
                if (request) {
                    this.sendq.delete(sequence);
                    request.callback(
                        new Response(sequence, {
                            sequence: sequence,
                            type: MessageType.RESPONSE,
                            status: 400,
                            data: "timeout",
                        })
                    );
                }
            }, 10000);

            let callback = (pkt: Response<S>) => {
                clearTimeout(tr);
                this.sendq.delete(sequence);
                if (pkt.status !== 200) {
                    message.error(pkt.data as string);
                    reject(pkt.data);
                    return;
                }
                resolve(pkt.data);
            };

            const sendData = {
                sequence: sequence,
                data: data,
            };

            this.send(JSON.stringify(sendData));
            this.sendq.set(sequence, new Request(sequence, data, callback));
        });
    }

    // 表示连接终止
    private onclose(reason: string) {
        info(reason);
        if (this.state == State.CLOSED) {
            return;
        }
        info("connection closed due to " + reason);
        this.ws = undefined;
        this.emit("close", reason);
        this.state = State.CLOSED;
    }

    private async errorHandler(error: Error) {
        if (this.state == State.CLOSED || this.state == State.CLOSEING) {
            return;
        }
        this.state = State.RECONNECTING;
        for (let index = 0; index < 10; index++) {
            await sleep(3);
            try {
                info("try to relogin");
                await this.connect();
                return;
            } catch (error) {
                warn(error as string);
            }
        }
        this.onclose("reconnect timeout");
    }

    private send(data: string): boolean {
        try {
            if (this.ws == null) {
                return false;
            }
            this.ws.send(data);
        } catch (error) {
            this.errorHandler(new Error("write timeout"));
            return false;
        }
        return true;
    }
}
