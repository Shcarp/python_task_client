import { useState } from "react";
import { Button, Form, Input, Space } from "antd";
import { useNavigate } from "react-router-dom";
import { sendRegister, checkUsernameExist } from "../../../api/user-center";
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
            const { msg } = await sendRegister(res.username, res.password);
            navigate("/user-center/login");
        } catch (error) {
            // message.error("登录失败");
        } finally {
            setLoading(false);
        }
    };

    const handleCheckUsername = async () => {
        const username = form.getFieldValue("username");
        const { data } = await checkUsernameExist(username);
        if (data) {
            return Promise.reject(new Error("用户名已存在"));
        }
        return Promise.resolve();
    }


    return (
        <Form form={form} layout="vertical">
            <Form.Item
                label="用户名"
                name="username"
                rules={[
                    {
                        required: true,
                        message: "请输入用户名",
                    },
                    () => ({
                        async validator(_, value) {
                            handleCheckUsername().then(() => {
                                return Promise.resolve();
                            }).catch((err) => {
                                return Promise.reject(err);
                            })
                        },
                    }),
                    
                ]}
            >
                <Input placeholder="请输入用户名"></Input>
            </Form.Item>
            <Form.Item
                label="密码"
                name="password"
                rules={[
                    {
                        required: true,
                        message: "请输入密码",
                    },
                ]}
            >
                <Input.Password placeholder="请输入密码" />
            </Form.Item>
            <Form.Item
                label="密码"
                name="password2"
                rules={[
                    {
                        required: true,
                        message: "请输入密码",
                    },
                    ({ getFieldValue }) => ({
                        validator(_, value) {
                            if (!value || getFieldValue("password") === value) {
                                return Promise.resolve();
                            }
                            return Promise.reject(new Error("两次密码不一致"));
                        },
                    }),
                ]}
            >
                <Input.Password placeholder="请输入密码" />
            </Form.Item>
            <Space>
                <Button type="primary" loading={loading} htmlType="submit" onClick={handleClick}>
                    注册
                </Button>
            </Space>
        </Form>
    );
};

export default Login;
