import { Button } from "antd";
import { client } from "../../utils/client/websocket";
import { useState } from "react";

client.connect();

const Test = () => {
    const [taskId, setTaskId] = useState('');
    const handleClick = async () => {
        try {
            let res = await client.send<string>("/task/add", { 
                name: "test", 
                run_count: 5, 
                trigger_type: "interval", 
                trigger_info: {
                    interval: 1,
                },
                execute_type: "Python",
                execute_info: {
                    key: "test.zip",
                    module: "py",
                    location: "local",
                    path: "/Users/shcarp/Desktop/python/python-task/__test__/testData/test.zip",
                    params: {a: 1, b: 2}
                },
            });
            setTaskId(res)
            console.log(res)
        } catch (error) {
            console.log(error);
        }
    };

    const handleGetList = async () => {
        try {
            const res = await client.send("/task/all", {});
            console.log(res);
        } catch (error) {
            console.log(error);
        }
    };

    const handleStart = async () => {
        try {
            const res = await client.send("/task/start", { task_id: taskId });
            console.log(res);
        } catch (error) {
            console.log(error);
        }
    };

    return (
        <div>
            <Button onClick={handleClick}>添加</Button>
            <Button onClick={handleGetList}>获取</Button>
            <Button onClick={handleStart}>开始</Button>
        </div>
    );
};

export default Test;
