import { websocketConn } from "@/utils/client/connect";
import { TaskWebSocketConnect } from "@/utils/client/task";
import { UserWebSocketConnect } from "@/utils/client/wechat";
import { Form, Input, Modal, message } from "antd";
import Router from "next/router";
import React, { useEffect, useImperativeHandle, useMemo, useState } from "react";

interface IProps {
    onOk?: () => void;
    onCancel?: () => void;
}

export interface IRef {
    open: () => void;
    close: () => void;
}



// eslint-disable-next-line react/display-name
export const AddWxName = React.forwardRef<IRef, IProps>((props, ref) => {
    const [open, setOpen] = useState(false);
    const [loading, setLoading] = useState(false);

    const userConn = useMemo(() => {
        if (!open) return;
        if (!websocketConn) {
            Router.push("/networkerror");
        } else {
            return new UserWebSocketConnect(websocketConn);
        }
    }, [open]);
    

    const [form] = Form.useForm();

    const handleOk = async () => {
        if (!userConn) return;
        try {
            setLoading(true);
            const res = await form?.validateFields();
            const success = await userConn.addWxUser(res);
            console.log(success);
            message.success("添加成功");
            setOpen(false);
        } catch (error) {
            console.log(error);
        } finally {
            setLoading(false);
        }
    };


    useImperativeHandle(ref, () => ({
        open: () => {
            setOpen(true);
        },
        close: () => {
            setOpen(false);
        },
    }));

    return (
        <Modal
            title="添加微信名称"
            destroyOnClose
            open={open}
            okButtonProps={{ loading: loading }}
            onCancel={() => setOpen(false)}
            onOk={handleOk}
        >
            <Form form={form} layout="vertical">
                <Form.Item label="微信名称" name="wx_name" rules={[{ required: true }]}>
                    <Input placeholder="请输入微信名称"></Input>
                </Form.Item>
            </Form>
        </Modal>
    );
});
