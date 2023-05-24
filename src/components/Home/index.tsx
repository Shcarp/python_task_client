import { ChangeEvent, useEffect, useRef, useState } from "react";
import { Button, Col, Empty, Input, Row, Space } from "antd";
import { IconFont } from "../Iconfont";
import { Task, TaskList } from "../List";
import { AddTask, IRef as AddTaskRef } from "../AddTask";
import { AddWxName, IRef as AddNameRef } from "../AddWxName";
import "./index.css";
import { getTaskWebSocketConn } from "@/app/connect";
import { debounce } from "lodash";

const taskConn = getTaskWebSocketConn();

export const Home = () => {
    const addTaskRef = useRef<AddTaskRef>(null);
    const addWxNameRef = useRef<AddNameRef>(null);
    const [loading, setLoading] = useState(false);
    const [data, setData] = useState<Task[]>([]);

    const [queryParam, setQueryParam] = useState({
        keyword: '',
    });

    const getTaskList = async () => {
        try {
            setLoading(true);
            const res = await taskConn.getTaskList({
                ...queryParam
            });
            setData(Array.isArray(res) ? res : []);
        } catch (error) {
            console.log(error)
        } finally {
            setLoading(false);
        }
    };

    const handleChange = debounce((e: ChangeEvent<HTMLInputElement>) => {
        setQueryParam({
            ...queryParam,
            keyword: e.target.value,
        });
    });

    const appendData = async () => {
        // setQueryParam({
        //     ...queryParam,
        // })
    }

    useEffect(() => {
        getTaskList();
    }, [queryParam]);

    useEffect(() => {
        getTaskList();
        const handle = (tasks: Task[]) => {
            setData(Array.isArray(tasks) ? tasks : []);
        }
        taskConn.on("task-list/update", handle)
        return () => {
            taskConn.off("task-list/update", handle)
        }
    }, []);

    return (
        <div className="home-content">
            <div>
                <Row justify="space-between">
                    <Col>
                        <Space>
                            <Button type="primary" onClick={() => addWxNameRef.current?.open()}>
                                添加微信名称
                            </Button>
                            <Input placeholder="搜索任务" onChange={handleChange}></Input>
                        </Space>
                    </Col>
                    <Col>
                        <Space>
                            <Button type="primary" onClick={() => addTaskRef.current?.open()}>
                                添加任务
                            </Button>
                            <Button type="primary" icon={<IconFont type="wx_message-buju" />}></Button>
                            <Button type="primary" icon={<IconFont type="wx_message-hengpai" />}></Button>
                        </Space>
                    </Col>
                </Row>
            </div>
            <div className="home-content-bottom">
                {
                    data.length !== 0 ? <TaskList loading={loading} data={data} appendData={appendData}></TaskList> : <Empty></Empty>
                }
                
            </div>
            <AddTask ref={addTaskRef} />
            <AddWxName ref={addWxNameRef} />
        </div>
    );
};
