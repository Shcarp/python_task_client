// export const ws = new WebSocket("ws://localhost:9673";

import { message } from "antd";
// import { WebSocket } from "ws";
import { generate_messageid } from "./utils";
import EventEmitter from "events";
import { info, error as logError, warn, debug } from "tauri-plugin-log-api";
import { PPush, PRequest, PRequestProps, PResponse, Kind } from "./protocol";
import { Events } from "./base";

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

// 请求
export class Request<T> {

    prequest: PRequest<T>;
    callback: (response: PResponse<any>) => void;
    constructor(prequest: PRequest<T>, callback: (response: PResponse<any>) => void) {
        this.prequest = prequest;
        this.callback = callback;
    }
}

type WebsocketEvent = {
    ["push"]: (data: PPush<any>) => void;
}
export class WebSocketConnect extends EventEmitter {

    on: Events<WebsocketEvent>["on"] = super.on;
    emit: Events<WebsocketEvent>["emit"] = super.emit;

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
        const data = JSON.parse(event.data);
        if (data.ctype === 'response' && data.status > 400) {
            this.ws?.close();
            info(`>>> error message ${data}`);
            return;
        }
        switch (data.ctype) {
            case 'ping':
                this.send("pong");
                break;
            case "pong":
                break;
            case "response":
                const response = new PResponse(data);

                const request = this.sendq.get(response.sequence);
                if (request) {
                    request.callback(response);
                } else {
                    info(`>>> request not found ${response.sequence}`);
                }
                break;
            case "push":
                const push = new PPush(data);
                this.emit("push", push);
                break;
            default:
                info(`>>> unknown message type ${data.type}`);
                break;
        }
    }

    protected request<T , S>(data: Omit<PRequestProps<T>, 'sequence'> ): Promise<S> {
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
                        new PResponse({
                            sequence: sequence,
                            status: 400,
                            data: "timeout",
                        })
                    );
                }
            }, 10000);

            let callback = (pkt: PResponse<S>) => {
                clearTimeout(tr);
                this.sendq.delete(sequence);
                if (pkt.status !== 200) {
                    message.error(pkt.data as string);
                    reject(pkt.data);
                    return;
                }
                resolve(pkt.data);
            };

            const sendData = new PRequest({
                ...data,
                sequence: sequence,
            });

            this.send(sendData.toJSON());
            this.sendq.set(sequence, new Request(sendData, callback));
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
