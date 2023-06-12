export type TaskType = "fixTime" | "intervalTime";

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

export const selectOptions = [
    {
        label: "进行中",
        value: TaskStatus.PROGRESS,
    },
    {
        label: "已完成",
        value: TaskStatus.COMPLETE,
    },
    {
        label: "已取消",
        value: TaskStatus.CANCEL,
    },
    {
        label: "已删除",
        value: TaskStatus.DELETE,
    },
    {
        label: "未开始",
        value: TaskStatus.NOSTARTED,
    },
]
