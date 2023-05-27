import { APP_BASEURL } from "../../app/config";
import { WebSocketConnect } from "./websocket";
import store from "../store";



export const connect = async (ip: string, port: number) => {
    return new Promise<WebSocketConnect>(async (resolve, reject) => {
        if (!websocketConn) {
            websocketConn = new WebSocketConnect(ip, port);
            websocketConn.on("connected", () => {
                resolve(websocketConn as WebSocketConnect);
            })
            websocketConn.on("closed", () => {
                websocketConn = null;
                reject();
            })
        } else {
            resolve(websocketConn);
        }
    })
}

export let websocketConn: WebSocketConnect | null;
