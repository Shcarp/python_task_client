import { Task } from "@/components/List";
import { APP_BASEURL } from "./config";
import { WebSocketConnect } from "./websocket";
import { Events } from "./base";
import { PPush } from "./protocol";

// let websocketConn: WebSocketConnect | null = null;
let taskWebsocketConn: TaskWebSocketConnect | null = null;
let userWebsocketConn: UserWebSocketConnect | null = null;

interface TaskListRequest {
    keyword?: string;
}

export const getTaskWebSocketConn = () => {
    if (!taskWebsocketConn) {
        const { ip, port } = JSON.parse(localStorage.getItem(APP_BASEURL) ?? "{}");
        taskWebsocketConn = new TaskWebSocketConnect(ip, port);
    }
    return taskWebsocketConn;
};

type TaskEvents = {
    ["task-item/state/update"]: (data: Task) => void;
    ["task-item/add"]: (data: Task) => void;
    ["task-list/update"]: (data: Task) => void;
};

export class TaskWebSocketConnect extends WebSocketConnect {
    on: Events<TaskEvents>["on"] = super.on;
    emit: Events<TaskEvents>["emit"] = super.emit;
    constructor(ip: string, port: number) {
        super(ip, port);

        this.on("push", (data: PPush<any>) => {
            console.log("push", data)
            this.emit(data.event, data.data);
        });
    }

    async addTask(data: Omit<Task, "id" | "status">) {
        return this.request({
            url: "/task/add",
            data,
        });
    }

    async editTask(data: Omit<Task, "status">) {
        return this.request({
            url: "/task/edit",
            data,
        });
    }

    async getTaskList(data?: TaskListRequest) {
        return this.request<Partial<TaskListRequest>, Task[]>({
            url: "/task/list",
            data: {},
        });
    }

    async deleteTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: "delete",
            },
        });
    }

    async startTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: "progress",
            },
        });
    }
 
    async stopTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: "cancel",
            },
        });
    }

}

export const getUserWebSocketConn = () => {
    if (!userWebsocketConn) {
        const { ip, port } = JSON.parse(localStorage.getItem(APP_BASEURL) ?? "{}");
        userWebsocketConn = new UserWebSocketConnect(ip, port);
    }
    return userWebsocketConn;
};

interface WxUser {
    name: string;
}

type WxEvent = {
    ["wechat-name/add"]: (data: string[]) => void;
}

export class UserWebSocketConnect extends WebSocketConnect {
    on: Events<WxEvent>["on"] = super.on;
    emit: Events<WxEvent>["emit"] = super.emit;
    constructor(ip: string, port: number) {
        super(ip, port);
        this.on("push", (data: PPush<any>) => {
            console.log("push", data)
            this.emit(data.event, data.data);
        });
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
            data: {}
        });
    }
}
