import { Button, Card, List, Space, Tag, message } from "antd";
import VirtualList from "rc-virtual-list";
import "./index.css";
import dayjs from "dayjs";
import { AddTask, IRef } from "../AddTask";
import { getTaskWebSocketConn } from "@/app/connect";
import { useRef, useState } from "react";
import { debounce } from "lodash";

export interface Task {
    id: number;
    name: string;
    type: TaskType;
    status: TaskStatus;
    time: number;
    member: string[];
    content: string;
}

/**
 * 固定时间任务和时间间隔触发任务
 */
export type TaskType = "fixTime" | "intervalTime";

const TaskTypeMap = {
    fixTime: {
        name: "固定时间任务",
        color: "blue",
    },
    intervalTime: {
        name: "时间间隔触发任务",
        color: "green",
    },
};

export type TaskStatus = "nostarted" | "progress" | "complete" | "cancel" | "delete";

const TaskStatusMap = {
    nostarted: {
        name: "未开始",
        color: "blue",
    },
    progress: {
        name: "进行中",
        color: "green",
    },
    complete: {
        name: "已完成",
        color: "yellow",
    },
    cancel: {
        name: "已取消",
        color: "red",
    },
    delete: {
        name: "已删除",
        color: "red",
    }
};

interface TaskListProps {
    data: Task[];
    loading: boolean;
    appendData?: () => void;
}

const taskConn = getTaskWebSocketConn();

export const TaskList: React.FC<TaskListProps> = ({ data, appendData, loading }) => {
    const addTaskRef = useRef<IRef>(null);

    const [loadings, setLoadings] = useState<boolean[]>([]);

    const handleStart = debounce(async (id: number) => {
        if (loadings[0]) return;
        try {
            setLoadings([true, loadings[1], loadings[2]])
            await taskConn.startTask({id});
            message.success("开始成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([false, loadings[1], loadings[2]])
        }
    }, 500);

    const handleRemove = debounce(async (id: number) => {
        if (loadings[1]) return;
        try {
            setLoadings([loadings[0], true, loadings[2]])
            await taskConn.deleteTask({id});
            message.success("移除成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([loadings[0], false, loadings[2]])
        }
    }, 500);

    const handleEdit = debounce((item: Task) => {
        addTaskRef.current?.open(item);
    }, 500);

    const handleCancel = debounce(async (id: number) => {
        if (loadings[2]) return;
        try {
            setLoadings([loadings[0], loadings[1], true])
            await taskConn.stopTask({id});
            message.success("取消成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([loadings[0], loadings[1], false])
        }
    }, 500);

    const renderCardTitle = (item: Task) => {
        return (
            <div className="list-item-header">
                <div className="list-item-header-top">{item.name}</div>
                <div className="list-item-header-bottom">
                    <Tag color={TaskTypeMap[item.type].color}>{TaskTypeMap[item.type].name}</Tag>
                    <Tag color={TaskStatusMap[item.status].color}>{TaskStatusMap[item.status].name}</Tag>
                </div>
            </div>
        );
    };

    const start = (item: Task) => {
        return (
            <a onClick={(e) => handleStart(item.id)}>开始</a>
        );
    };

    const remove = (item: Task) => {
        return (
            <a onClick={(e) => handleRemove(item.id)}> 移除</a>
        );
    };

    const cancel = (item: Task) => {
        return (
            <a onClick={(e) => handleCancel(item.id) }> 取消 </a>
        );
    };

    const edit = (item: Task) => {
        return (
            <a onClick={(e) => { handleEdit(item)}}>编辑 </a>
        );
    };

    const renderCardExtra = (item: Task) => {
        const btns = {
            nostarted: [
                start,
                remove,
                edit,
            ],
            progress: [
                cancel,
            ],
            complete: [
                start,
                edit,
            ],
            cancel: [
                start,
                edit,
            ],
            delete: []
        };

        return (
            <div className="list-item-extra">
                <Space>{btns[item.status].map((btn) => btn(item))}</Space>
            </div>
        );
    };

    const renderContent = (item: Task) => {
        const title = item.type === "fixTime" ? "任务时间" : "任务间隔";
        const time = item.type === "fixTime" ? dayjs(item.time).format("HH:MM") : `${item.time}秒`;
        return (
            <div className="list-item-content">
                <div className="list-item-content-top">
                    <div>
                        <span>{`${title}：`}</span>
                        <span>{time}</span>
                    </div>
                    <div>
                        <span>任务成员：</span>
                        <span>{item.member.join("，")}</span>
                    </div>
                    <div>
                        <span>任务内容：</span>
                        <span>{item.content}</span>
                    </div>
                </div>
                <AddTask ref={addTaskRef} />
            </div>
        );
    };

    const onScroll = (e: React.UIEvent<HTMLElement, UIEvent>) => {
        if (e.currentTarget.scrollHeight - e.currentTarget.scrollTop === 460) {
            appendData && appendData();
        }
      };

    return (
        <List dataSource={data} loading={loading}>
            <VirtualList data={data.filter((item) => item.status !== 'delete')} height={460} itemHeight={100} itemKey={"id"} onScroll={onScroll}>
                {(item: Task) => {
                    return (
                        <List.Item>
                            <Card
                                style={{ width: "100%" }}
                                title={renderCardTitle(item)}
                                type="inner"
                                extra={renderCardExtra(item)}
                            >
                                {renderContent(item)}
                            </Card>
                        </List.Item>
                    );
                }}
            </VirtualList>
        </List>
    );
};
