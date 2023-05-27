import { websocketConn } from "@/utils/client/connect";
import { PPush } from "@/utils/client/protocol";
import { MessageData, TaskWebSocketConnect } from "@/utils/client/task";
import { List, Modal } from "antd";
import dayjs from "dayjs";
import { debounce } from "lodash";
import Router from "next/router";
import VirtualList from "rc-virtual-list";
import React, { useImperativeHandle } from "react";
import { useEffect, useState } from "react";

const info_type = {
    0: {
        name: "success",
        color: "green",
    },
    1: {
        name: "error",
        color: "red",
    },
    2: {
        name: "warn",
        color: "yellow",
    },
};

// eslint-disable-next-line react/display-name
const Message = React.forwardRef((_, ref) => {
    const [open, setOpen] = useState(false);
    const [data, setData] = useState<MessageData[]>([]);
    const [height, setHeight] = useState(0);
    const [width, setWidth] = useState(0);

    useEffect(() => {
        const handleData = (data: PPush<string>) => {
            setData((prev) => {
                return [
                    ...prev,
                    {
                        status: data.status,
                        sendTime: data.sendTime,
                        msg: data.data,
                    },
                ];
            });
        };

        if (websocketConn) {
            const taskConn = new TaskWebSocketConnect(websocketConn);
            taskConn.on("info", handleData);
            return () => {
                taskConn.off("info", handleData);
            };
        } else {
            Router.push("/networkerror");
        }
       
    }, []);

    useEffect(() => {
        const handleHeight = debounce(() => {
            const height = document.documentElement.clientHeight - 100;
            const width = document.documentElement.clientWidth - 100;
            setHeight(height);
            setWidth(width);
        }, 500);
        // 监听窗口变化
        document.addEventListener("resize", handleHeight);
        handleHeight();
        return () => {
            document.removeEventListener("resize", handleHeight);
        };
    }, []);

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
            title="消息"
            open={open}
            width={width}
            style={{ height: 400, top: 10 }}
            onCancel={() => setOpen(false)}
            footer={null}
        >
            <List dataSource={data}>
                <VirtualList data={data} height={height} itemHeight={20} itemKey={(item: MessageData) => item.sendTime}>
                    {(item: MessageData) => {
                        const { name, color } = info_type[item.status];
                        return (
                            <List.Item color={color}>
                                <span style={{ color: color }}>[{name}]</span>{" "}
                                {`[${dayjs(item.sendTime * 1000).format("YYYY-MM-DD HH:mm")}] ${item.msg}`}
                            </List.Item>
                        );
                    }}
                </VirtualList>
            </List>
        </Modal>
    );
});

export default Message;
