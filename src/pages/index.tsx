import { useEffect, useRef } from "react";
import Router from "next/router"
import lottie from "lottie-web";
import { Card, Col, Row, message } from "antd";
import { APPTYPE_NAME, APP_API_DEAFULT_PORT, APP_BASEURL } from "@/app/config";
import { connect } from "@/utils/client/connect";

export enum TaskType {
    Local = "local",
    Remote = "remote",
}

export default function Index() {
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
                    localStorage.setItem(APPTYPE_NAME, TaskType.Local);
                    await connect("127.0.0.1", APP_API_DEAFULT_PORT);
                    Router.push("/task-home");
                    break;
                } catch (error) {
                    message.error("连接失败");
                }
            case TaskType.Remote:
                localStorage.setItem(APPTYPE_NAME, TaskType.Remote);
                Router.push("/connect");
                break;
            default:
                break;
        }
    }

    return (
        <Row className="row" justify="space-around">
            <Col className="card" onClick={() => handleClick(TaskType.Local)}>
                <Card  hoverable cover={<div className="local" ref={localRef}></div>}>
                    <Card.Meta title="本地" />
                </Card>
            </Col>
            <Col className="card" onClick={() => handleClick(TaskType.Remote)}>
                <Card hoverable cover={<div className="remote" ref={remoteRef}></div>}>
                    <Card.Meta title="远程" />
                </Card>
            </Col>
        </Row>
    );
}
