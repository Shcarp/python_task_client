import { useEffect, useState } from "react";
import { Button, Form, Input, Space, message } from "antd";
import { useNavigate } from "react-router-dom";
import { AUTH_TOKEN } from "../../../config";
import { sendLogin } from "../../../api/user-center";
import getStore from "../../../utils/store";

const { useForm } = Form;

export const Login = () => {
    const navigate = useNavigate();
    const [loading, setLoading] = useState(false);
    const [form] = useForm();

    const handleClick = async () => {
        try {
            setLoading(true);
            const res = await form.validateFields();
            const store = await getStore();
            await store.set("username", res.username);
            await store.set("password", res.password);
            const responseData = await sendLogin(res.username, res.password);
            await store.set(AUTH_TOKEN, responseData.data.token);
            await store.set("userInfo", responseData.data);
            message.success(responseData.msg);
            navigate("/");
        } catch (error) {
            // message.error("登录失败");
        } finally {
            setLoading(false);
        }
    };

    const handleRegister = async () => {
        navigate("/user-center/register");
    }
    
    useEffect(() => {
        (async () => {
            const store = await getStore();
            const username = await store.get("username");
            const password = await store.get("password");
            if (username && password) {
                form.setFieldsValue({
                    username,
                    password,
                });
            }
        })();
    }, []);

    return (
        <Form form={form} layout="vertical">
            <Form.Item label="用户名" name="username" rules={[
                    {
                        required: true,
                        message: "请输入密码",
                    },
                ]}>
                <Input placeholder="请输入用户名"></Input>
            </Form.Item>
            <Form.Item label="密码" name="password" rules={[
                    {
                        required: true,
                        message: "请输入密码",
                    },
                ]}>
                <Input.Password placeholder="请输入密码" />
            </Form.Item>

            <Space align="center">
                <Button type="primary" loading={loading} htmlType="submit" onClick={handleClick}>
                    登录
                </Button>
                <Button type="default" htmlType="submit" onClick={handleRegister}>
                    注册
                </Button>
            </Space>
        </Form>
    );
};

export default Login;
