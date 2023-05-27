import EventEmitter from "events";
import { Events } from "./base";
import { PPush } from "./protocol";
import { WebSocketConnect } from "./websocket";

interface WxUser {
    name: string;
}

type WxEvent = {
    ["wechat-name/add"]: (data: string[]) => void;
};

export class UserWebSocketConnect extends EventEmitter {
    on: Events<WxEvent>["on"] = super.on;
    emit: Events<WxEvent>["emit"] = super.emit;

    private conn: WebSocketConnect;

    constructor(conn: WebSocketConnect) {
        super();
        this.conn = conn;
        this.conn.on("push", (data: PPush<any>) => {
            this.emit(data.event, data);
        });
    }

    private get request() {
        return this.conn.request.bind(this.conn);
    }


    async addWxUser(data: WxUser) {
        return this.request({
            url: "/wxuser/add",
            data,
        });
    }

    async getAllWxUser(): Promise<string[]> {
        return this.request({
            url: "/wxuser/list",
            data: {},
        });
    }
}
