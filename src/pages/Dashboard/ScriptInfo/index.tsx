import { Space, Image, Alert, Descriptions, Table } from "antd";
import { IconFont } from "../../../components/Iconfont";

import styles from "./index.module.less";
import { useEffect, useMemo, useState } from "react";
import { getScriptDetail } from "../../../api/script";
import { ScriptDetail } from "../../../api/script.type";
import dayjs from "dayjs";

interface ScriptInfoProps {
    scriptUid?: string;
}

export const ScriptInfo: React.FC<ScriptInfoProps> = ({ scriptUid }) => {
    const [currentSelect, setCurrentSelect] = useState<ScriptDetail>();

    const column = [
        {
            title: "版本",
            dataIndex: "scriptVersion",
            key: "scriptVersion",
        },
        {
            title: "最近更新时间",
            dataIndex: "created_at",
            key: "created_at",
            render: (text: string) => {
                return <span>{dayjs(text).format("YYYY-MM-DD")}</span>;
            }
        },
        {
            title: "操作",
            dataIndex: "operation",
            key: "operation",
            render: (text: string, record: any) => {
                return (
                    <Space>
                        <span>使用</span>
                    </Space>
                );
            }
        }
    ]

    useEffect(() => {
        if (!scriptUid) return;
        getScriptDetail(scriptUid).then((res) => {
            setCurrentSelect(res.data);
        });
    }, [scriptUid]);

    return (
        <>
            <Space direction="vertical" size="middle" style={{ display: "flex" }}>
                <Image
                    width={300}
                    preview={false}
                    alt="example"
                    src={"https://gw.alipayobjects.com/zos/rmsportal/JiqGstEfoWAOHiTxclqi.png"}
                ></Image>
                <Space>
                    <span className={styles.likeAndFItem}>
                        <IconFont className={styles.icon} type="wx_message-xihuan"></IconFont>
                        {currentSelect?.likes}
                    </span>
                    <span className={styles.likeAndFItem}>
                        <IconFont className={styles.icon} type="wx_message-shoucang"></IconFont>
                        {currentSelect?.favorites}
                    </span>
                </Space>
                {/* <Alert description={currentSelect?.scriptDetailedDescription} type="info" /> */}
                <Descriptions column={1} title={currentSelect?.scriptName} layout="horizontal">
                    <Descriptions.Item label="简介">{currentSelect?.scriptDescription}</Descriptions.Item>
                    <Descriptions.Item label="详情">{currentSelect?.scriptDetailedDescription}</Descriptions.Item>
                </Descriptions>
                <span>版本列表</span>
                <Table columns={column} dataSource={currentSelect?.items}></Table>
            </Space>
        </>
    );
};
