import { APP_BASEURL } from "@/app/config";
import { Button, Form, Input } from "antd";
import Router from "next/router";

const Connect = () => {
    const [from] = Form.useForm();
    const handleOk = async () => {
        try {
            const res = await from?.validateFields();
            localStorage.setItem(APP_BASEURL, JSON.stringify({ip: res.ip, port: res.port}));
            Router.push("/task");
        } catch (error) {
            console.log(error);
        }
    };

    return (
        <div>
            <Form form={from}>
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
