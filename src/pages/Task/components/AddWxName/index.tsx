import { Form, Input, Modal, message } from "antd";
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

    const [form] = Form.useForm();

    const handleOk = async () => {
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
