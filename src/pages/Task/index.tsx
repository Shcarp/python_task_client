import React, { useEffect, useRef, useState } from "react";
import { Button, Col, Empty, Form, Input, Row, Select, Space, Statistic, message } from "antd";
import { IRef as AddTaskRef, TaskAction } from "./components/AddTask";
import { IRef as AddNameRef, AddWxName } from "./components/AddWxName";
import { debounce } from "lodash";
import { Task, TaskStatus } from "./type";
import Message from "./components/Message";
import { TaskList } from "./components/List";
import styles from './index.module.less'

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


const TaskComp = () => {
    const addTaskRef = useRef<AddTaskRef>(null);
    const addWxNameRef = useRef<AddNameRef>(null);
    const messageRef = useRef<any>(null);

    const [loading, setLoading] = useState(false);
    const [data, setData] = useState<Task[]>([]);

    const [runingTotal, setRuningTotal] = useState(0);
    const [wait, setWait] = useState(0);

    const [queryParam, setQueryParam] = useState({
        keyword: "",
        status: 0,
    });

    const getTaskList = async () => {
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
        (
            async () => {
                const { appWindow } = await import("@tauri-apps/api/window");
                appWindow.setTitle(`微信群发助手(阻塞的任务数量：${wait}/当前进行中的任务数量：${runingTotal})`);
            }
        )()
    }, [runingTotal, wait]);

    return (
        <div className={styles["home-content"]}>
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
            <div className={styles["home-content-bottom"]}>
                {data.length !== 0 ? <TaskList loading={loading} data={data}></TaskList> : <Empty></Empty>}
            </div>
            <TaskAction ref={addTaskRef} />
            <AddWxName ref={addWxNameRef} />
            <Message ref={messageRef} />
        </div>
    );
};

export default TaskComp;
