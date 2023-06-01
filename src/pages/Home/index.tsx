import React, { useEffect, useRef } from "react";
import lottie from "lottie-web";
import { useNavigate } from "react-router-dom";
import { Card, Col, Row, message } from "antd";
import styles from "./index.module.less"

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
                    // localStorage.setItem(APPTYPE_NAME, TaskType.Local);
                    // await connect("127.0.0.1", APP_API_DEAFULT_PORT);
                    // Router.push("/task-home");
                    // break;
                    navigate("/task");
                } catch (error) {
                    message.error("连接失败");
                }
                break;
            case TaskType.Remote:
                navigate("/connect");
            default:
                break;
        }
    }

    return (
        <Row className={styles.row} justify="space-around">
            <Col className={styles.card} onClick={() => handleClick(TaskType.Local)}>
                <Card  hoverable cover={<div className={styles.local} ref={localRef}></div>}>
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
