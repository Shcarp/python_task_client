import { DatePicker, Form, Input, Modal, Select, message } from "antd";
import React, { useEffect, useImperativeHandle, useMemo } from "react";
import { useState } from "react";
import dayjs from "dayjs";
import { debounce } from "lodash";
import { Task, TaskStatus } from "../../type";

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
        // if (!userConn) return;
        // const res = await userConn.getAllWxUser();
        // setMemberList(Array.isArray(res) ? res : []);
    };

    const handleOk = debounce(async () => {
        setOpen(false);
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
                        defaultValue="fixTime"
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
