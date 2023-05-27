import EventEmitter from "events";
import { Events } from "./base";
import { PPush } from "./protocol";
import { WebSocketConnect } from "./websocket";
import { connect, websocketConn } from "./connect";

export interface MessageData {
    sendTime: number;
    msg: string;
    status: 0 | 1 | 2;
}

export interface Task {
    id: number;
    name: string;
    type: TaskType;
    status: TaskStatus;
    time: number;
    member: string[];
    content: string;
}

export interface TaskListResp {
    list: Task[];
    running_count: number;
}

export enum TaskStatus {
    NOSTARTED = 1,
    PROGRESS = 2,
    COMPLETE = 3,
    CANCEL = 4,
    DELETE = 5,
}

interface TaskListRequest {
    keyword?: string;
    status?: TaskStatus;
}

/**
 * 固定时间任务和时间间隔触发任务
 */
export type TaskType = "fixTime" | "intervalTime";

type TaskEvents = {
    ["task-item/state/update"]: (data: PPush<Task[]>) => void;
    ["task-item/add"]: (data: PPush<Task[]>) => void;
    ["task-list/update"]: (data: PPush<Task[]>) => void;
    ["info"]: (data: MessageData) => void;
    ["block_num"]: (data: PPush<number>) => void;
};

export class TaskWebSocketConnect extends EventEmitter {
    on: Events<TaskEvents>["on"] = super.on;
    emit: Events<TaskEvents>["emit"] = super.emit;

    private conn: WebSocketConnect;

    private get request() {
        return this.conn.request.bind(this.conn);
    }

    constructor(conn: WebSocketConnect) {
        super();
        this.conn = conn;
        this.conn.on("push", (data: PPush<any>) => {
            this.emit(data.event, data);
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
        return this.request<Partial<TaskListRequest>, TaskListResp>({
            url: "/task/list",
            data: data ?? {},
        });
    }

    async deleteTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: TaskStatus.DELETE,
            },
        });
    }

    async startTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: TaskStatus.PROGRESS,
            },
        });
    }

    async stopTask(data: Pick<Task, "id">) {
        return this.request({
            url: "/task/update",
            data: {
                ...data,
                status: TaskStatus.CANCEL,
            },
        });
    }
}
