import { useEffect, useRef, useState } from "react";
import { Button, Col, Empty, Form, Input, Row, Select, Space, Statistic, message } from "antd";
import { IconFont } from "../Iconfont";
import { AddTask, IRef as AddTaskRef } from "../AddTask";
import { AddWxName, IRef as AddNameRef } from "../AddWxName";
import { debounce } from "lodash";
import Message from "../Message";
import { PPush } from "@/utils/client/protocol";
import { TaskList } from "../List";
import { Task, TaskListResp, TaskStatus, TaskWebSocketConnect } from "@/utils/client/task";
import "./index.css";
import { websocketConn } from "@/utils/client/connect";
import Router from "next/router";

const selectOptions = [
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


export const Home = () => {
    const addTaskRef = useRef<AddTaskRef>(null);
    const addWxNameRef = useRef<AddNameRef>(null);
    const messageRef = useRef<any>(null);

    const [taskConn, setTaskConn] = useState<TaskWebSocketConnect>();

    const [loading, setLoading] = useState(false);
    const [data, setData] = useState<Task[]>([]);

    const [runingTotal, setRuningTotal] = useState(0);
    const [wait, setWait] = useState(0);

    const [queryParam, setQueryParam] = useState({
        keyword: "",
        status: 0,
    });

    const getTaskList = async () => {
        if (!taskConn) { return }
        try {
            setLoading(true);
            const res = await taskConn.getTaskList({
                ...queryParam,
            });
            setData(Array.isArray(res.list) ? res.list : []);
            setRuningTotal(res.running_count);
        } catch (error) {
            console.log(error);
        } finally {
            setLoading(false);
        }
    };

    const handleChange = debounce((value, values) => {
        setQueryParam({
            ...queryParam,
            keyword: values.keyword,
            status: values?.status ?? 0,
        });
    });


    useEffect(() => {
        getTaskList();
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [queryParam]);

    useEffect(() => {
        const handle = (tasks: PPush<TaskListResp>) => {
            // 运行中的数量
            setRuningTotal(tasks.data.running_count);
            setData(Array.isArray(tasks.data.list) ? tasks.data.list : []);
        };

        const handleNum = (data: PPush<number>) => {
            setWait(data.data);
        };

        if (websocketConn) {
            const taskConn = new TaskWebSocketConnect(websocketConn);
            taskConn.on("task-list/update", handle);

            taskConn.on("block_num", handleNum);
            setTaskConn(taskConn);
            return () => {
                taskConn.off("task-list/update", handle);
                taskConn.off("block_num", handleNum);
            };
            
        } else {
            Router.push("/networkerror");
        }
    }, []);

    useEffect(() => {
        if (taskConn) {
            getTaskList();
        }
    }, [taskConn]);


    useEffect(() => {
        (
            async () => {
                const { appWindow } = await import("@tauri-apps/api/window");
                appWindow.setTitle(`微信群发助手(阻塞的任务数量：${wait}/当前进行中的任务数量：${runingTotal})`);
            }
        )()
    }, [runingTotal, wait]);

    return (
        <div className="home-content">
            <Row justify="space-between">
                <Col flex={1}>
                    <Form onValuesChange={handleChange}>
                        <Space align="start">
                            <Button type="primary" onClick={() => addWxNameRef.current?.open()}>
                                添加微信名称
                            </Button>
                            <Form.Item name="keyword">
                                <Input size="middle" placeholder="搜索任务"></Input>
                            </Form.Item>    
                            
                            <Form.Item name="status">
                                <Select showSearch allowClear size="middle" placeholder="任务状态" options={selectOptions}></Select>
                            </Form.Item>
                            
                        </Space>
                    </Form>
                </Col>
                <Col flex={1} style={{ textAlign: "right" }}>
                    <Space align="baseline">
                        {/* <Statistic title={null} value={wait} suffix={`/ ${runingTotal}`} /> */}
                        <Button type="primary" onClick={() => addTaskRef.current?.open()}>
                            添加任务
                        </Button>
                        <Button onClick={() => messageRef.current?.open()}>LOG</Button>
                    </Space>
                </Col>
            </Row>
            <div className="home-content-bottom">
                {data.length !== 0 ? <TaskList loading={loading} data={data}></TaskList> : <Empty></Empty>}
            </div>
            <AddTask ref={addTaskRef} />
            <AddWxName ref={addWxNameRef} />
            <Message ref={messageRef} />
        </div>
    );
};
