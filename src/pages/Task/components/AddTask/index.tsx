import { DatePicker, Form, Input, Modal, Select, message } from "antd";
import React, { useEffect, useImperativeHandle, useMemo } from "react";
import { useState } from "react";
import dayjs from "dayjs";
import { debounce } from "lodash";
import { Task, TaskStatus } from "../../type";
import { PushData, client } from "../../../../utils/client/websocket";

interface IProps {
    onOk?: () => void;
    onCancel?: () => void;
}

export interface IRef {
    open: (item?: Task) => void;
    close: () => void;
}

// eslint-disable-next-line react/display-name
export const TaskAction = React.forwardRef<IRef, IProps>((props, ref) => {
    const [open, setOpen] = useState(false);
    const [opentype, setOpenType] = useState("add");
    const [form] = Form.useForm();

    const [loading, setLoading] = useState(false);

    const [taskType, setTaskType] = useState("fixTime");
    const [memberList, setMemberList] = useState<string[]>([]);
    

    const geWxUserList = async () => {
        const res = await client.send<string[]>("/wxuser/list", {});
        setMemberList(res);
    };

    const handleOk = debounce(async () => {
        try {
            const res = await form.validateFields();
            setLoading(true);
            const requestBody = {
                ...res,
                time: taskType === "fixTime" ? res.time.valueOf() : res.time,
            };
            switch (opentype) {
                case "add":
                    await client.send("/task/add", requestBody);
                    break;
                case "edit":
                    await client.send("/task/edit", requestBody);
                    break;
                default:
                    break;
            } 
            message.success("添加成功");
            setOpen(false);
        } catch (error) {
            console.log(error);
            message.error("添加失败");
        } finally {
            setLoading(false);
        }
    }, 500);

    const handleClose = () => {
        form.resetFields();
        setOpen(false);
    };

    useEffect(() => {
        if (open) {
            geWxUserList();
        }
    }, [open]);

    useEffect(() => {
        const handleUserAdd = (data: PushData<string[]>) => {
            setMemberList(data.data);
        }
        client.on("wechat-name/add", handleUserAdd);
        return () => {
            client.off("wechat-name/add", handleUserAdd);
        }
    }, []);

    useImperativeHandle(ref, () => ({
        open: (item?: Task) => {
            if (item) {
                setOpenType("edit");
                setTaskType(item.type);
                form.setFieldsValue({
                    ...item,
                    time: item.type === "fixTime" ? dayjs(item.time) : item.time,
                });
            }
            setOpen(true);
        },
        close: handleClose,
    }));

    return (
        <Modal
            destroyOnClose
            onOk={handleOk}
            okButtonProps={{ loading: loading }}
            title={opentype === "add" ? "添加任务" : "编辑任务"}
            open={open}
            style={{ top: 20 }}
            onCancel={handleClose}
        >
            <Form
                form={form}
                layout="vertical"
                initialValues={{
                    type: "fixTime",
                    id: 0,
                    status: TaskStatus.NOSTARTED,
                }}
            >
                <Form.Item hidden label="任务ID" name="id">
                    {" "}
                    <Input></Input>{" "}
                </Form.Item>
                <Form.Item hidden label="任务状态" name="status">
                    <Input></Input>
                </Form.Item>
                <Form.Item label="任务名称" name="name" rules={[{ required: true }]}>
                    <Input placeholder="请输入任务名称"></Input>
                </Form.Item>
                <Form.Item label="任务类型" name="type">
                    <Select
                        placeholder="请选择任务类型"
                        onChange={(value) => setTaskType(value)}
                    >
                        <Select.Option value="fixTime">定时任务</Select.Option>
                        <Select.Option value="intervalTime">间隔任务</Select.Option>
                    </Select>
                </Form.Item>
                <Form.Item label="任务时间" name="time">
                    {taskType === "fixTime" ? (
                        <DatePicker picker="time" />
                    ) : (
                        <Input placeholder="请输入任务间隔"></Input>
                    )}
                </Form.Item>
                <Form.Item label="信息内容" name="content">
                    <Input.TextArea placeholder="请输入信息内容" />
                </Form.Item>
                <Form.Item label="成员" name="member">
                    <Select
                        allowClear
                        mode="multiple"
                        placeholder="请选择成员"
                        options={memberList?.map((item) => {
                            return {
                                label: item,
                                value: item,
                            };
                        })}
                    ></Select>
                </Form.Item>
            </Form>
        </Modal>
    );
});
