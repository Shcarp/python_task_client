import React from "react";
import { Button, Form, Input } from "antd";
import { useNavigate } from "react-router-dom";
import { APP_API_DEAFULT_PORT } from "../../config";

const Connect = () => {
    const navigate = useNavigate();
    const [from] = Form.useForm();
    const handleOk = async () => {
        try {
            const res = await from?.validateFields();
            navigate("/task");
        } catch (error) {
            console.log(error);
        }
    };

    return (
        <div>
            <Form form={from} initialValues={{
                port: APP_API_DEAFULT_PORT
            }}>
                <Form.Item name="ip" rules={[{ required: true }]}>
                    <Input placeholder="请输入IP"></Input>
                </Form.Item>
                <Form.Item name="port" rules={[{ required: true }]}>
                    <Input placeholder="请输入端口"></Input>
                </Form.Item>
            </Form>
            <Button onClick={handleOk}>下一步</Button>
        </div>
    );
};

export default Connect;
