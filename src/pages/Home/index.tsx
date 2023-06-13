import { Card, Col, Row, message } from "antd";
import React, { useEffect, useRef } from "react";

import { PushData, WebsocketClient, client } from "../../utils/client/websocket";
import { dialog } from "@tauri-apps/api";
import { info, error, warn } from "tauri-plugin-log-api"
import lottie from "lottie-web";
import styles from "./index.module.less";
import { useNavigate } from "react-router-dom";
import dayjs from "dayjs";

export enum TaskType {
    Local = "local",
    Remote = "remote",
}

export default function Index() {
    const navigate = useNavigate();
    const localRef = useRef(null);
    const remoteRef = useRef(null);

    useEffect(() => {
        if (!localRef.current) {
            return;
        }
        const am = lottie.loadAnimation({
            container: localRef.current,
            renderer: "svg",
            loop: true,
            autoplay: true,
            path: "/lottie/local.json", // the path to the animation json
        });

        return () => {
            am.destroy();
        };
    }, []);

    useEffect(() => {
        if (!remoteRef.current) {
            return;
        }
        const am = lottie.loadAnimation({
            container: remoteRef.current, // the dom element that will contain the animation
            renderer: "svg",
            loop: true,
            autoplay: true,
            path: "/lottie/remote.json", // the path to the animation json
        });

        return () => {
            am.destroy();
        };
    }, []);

    const handleClick = async (type: TaskType) => {
        switch (type) {
            case TaskType.Local:
                try {
                    await client.connect();
                    client.on("info", (data: PushData<string>) => {
                        const map = {
                            0: info,
                            1: error,
                            2: warn
                        }

                        map[data.status](`[${dayjs(data.sendTime * 1000).format("YYYY-MM-DD HH:mm")}] ${data.data}`)
                    })
                    navigate("/task");
                } catch (error) {
                    dialog.confirm("连接失败", { title: "提示", type: "error"})
                } 
                break;
            case TaskType.Remote:
                navigate("/connect");
            default:
                break;
        }
    };

    return (
        <Row className={styles.row} justify="space-around">
            <Col className={styles.card} onClick={() => handleClick(TaskType.Local)}>
                <Card hoverable cover={<div className={styles.local} ref={localRef}></div>}>
                    <Card.Meta title="本地" />
                </Card>
            </Col>
            <Col className={styles.card} onClick={() => handleClick(TaskType.Remote)}>
                <Card hoverable cover={<div className={styles.remote} ref={remoteRef}></div>}>
                    <Card.Meta title="远程" />
                </Card>
            </Col>
        </Row>
    );
}
