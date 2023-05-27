import { Card, List, Space, Tag, message } from "antd";
import VirtualList from "rc-virtual-list";

import dayjs from "dayjs";
import { AddTask, IRef } from "../AddTask";
import { useEffect, useRef, useState } from "react";
import { debounce } from "lodash";
import "./index.css";
import { Task, TaskStatus, TaskWebSocketConnect } from "@/utils/client/task";
import { websocketConn } from "@/utils/client/connect";
import Router from "next/router";

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

// export type TaskStatus = "nostarted" | "progress" | "complete" | "cancel" | "delete";

const TaskStatusMap = {
    [TaskStatus.NOSTARTED]: {
        name: "未开始",
        color: "blue",
    },
    [TaskStatus.PROGRESS]: {
        name: "进行中",
        color: "green",
    },
    [TaskStatus.COMPLETE]: {
        name: "已完成",
        color: "yellow",
    },
    [TaskStatus.CANCEL]: {
        name: "已取消",
        color: "red",
    },
    [TaskStatus.DELETE]: {
        name: "已删除",
        color: "red",
    },
};

interface TaskListProps {
    data: Task[];
    loading: boolean;
    appendData?: () => void;
}

export const TaskList: React.FC<TaskListProps> = ({ data, appendData, loading }) => {
    const addTaskRef = useRef<IRef>(null);
    const [height, setHeight] = useState(0);
    const [loadings, setLoadings] = useState<boolean[]>([]);

    const [taskConn, setTaskConn] = useState<TaskWebSocketConnect>();

    const handleStart = debounce(async (id: number) => {
        if (!taskConn) return;
        if (loadings[0]) return;
        try {
            setLoadings([true, loadings[1], loadings[2]]);
            await taskConn.startTask({ id });
            message.success("开始成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([false, loadings[1], loadings[2]]);
        }
    }, 500);

    const handleRemove = debounce(async (id: number) => {
        if (!taskConn) return;
        if (loadings[1]) return;
        try {
            setLoadings([loadings[0], true, loadings[2]]);
            await taskConn.deleteTask({ id });
            message.success("移除成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([loadings[0], false, loadings[2]]);
        }
    }, 500);

    const handleEdit = debounce((item: Task) => {
        addTaskRef.current?.open(item);
    }, 500);

    const handleCancel = debounce(async (id: number) => {
        if (!taskConn) return;
        if (loadings[2]) return;
        try {
            setLoadings([loadings[0], loadings[1], true]);
            await taskConn.stopTask({ id });
            message.success("取消成功");
        } catch (error) {
            console.log(error);
        } finally {
            setLoadings([loadings[0], loadings[1], false]);
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
        return <a key={`${item.id}+0`} onClick={(e) => handleStart(item.id)}>开始</a>;
    };

    const remove = (item: Task) => {
        return <a key={`${item.id}+1`} onClick={(e) => handleRemove(item.id)}> 移除</a>;
    };

    const cancel = (item: Task) => {
        return <a key={`${item.id}+2`} onClick={(e) => handleCancel(item.id)}> 取消 </a>;
    };

    const edit = (item: Task) => {
        return (
            <a
                key={`${item.id}+3`}
                onClick={(e) => {
                    handleEdit(item);
                }}
            >
                编辑{" "}
            </a>
        );
    };

    const renderCardExtra = (item: Task) => {
        const btns = {
            [TaskStatus.NOSTARTED]: [start, remove, edit],
            [TaskStatus.PROGRESS]: [cancel],
            [TaskStatus.COMPLETE]: [start, edit],
            [TaskStatus.CANCEL]: [start, remove, edit],
            [TaskStatus.DELETE]: [edit],
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

    useEffect(() => {
        const handleHeight = debounce(() => {
            const height = document.documentElement.clientHeight - 100;
            setHeight(height);
        }, 500);
        // 监听窗口变化
        document.addEventListener("resize", handleHeight);
        handleHeight();
        return () => {
            document.removeEventListener("resize", handleHeight);
        };
    }, []);

    useEffect(() => {
        if (websocketConn) {
            const taskConn = new TaskWebSocketConnect(websocketConn);
            setTaskConn(taskConn);
        } else {
            Router.push("/networkerror");
        }
    }, []);

    return (
        <List dataSource={data} loading={loading}>
            <VirtualList
                data={data}
                height={height}
                itemHeight={100}
                itemKey={(item: Task) => item.id}
                onScroll={onScroll}
            >
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
