import { window as tauriWindow } from "@tauri-apps/api";
import React, { Suspense, useEffect } from "react";
import { RouterProvider } from "react-router-dom";
import { Loading } from "./components/Loading";
import { router } from "./router";
import { Badge, ConfigProvider } from "antd";
import { client } from "./utils/client/websocket";
import "dayjs/locale/zh-cn";
import zhCN from "antd/locale/zh_CN";
import "antd/dist/reset.css";
import "./App.less";


const App = () => {
    const [executeState, setExecuteState] = React.useState(false);

    useEffect(() => {
        const handleMouseWheel = async (e: any) => {
            await tauriWindow.appWindow.startDragging();
            e.preventDefault();
        };
        document.querySelector("#drag")?.addEventListener("mousedown", handleMouseWheel);
        return () => {
            document.querySelector("#drag")?.removeEventListener("mousedown", handleMouseWheel);
        };
    }, []);

    useEffect(() => {
        const handleConnect = () => setExecuteState(true);
        const handleError = () => setExecuteState(false);
        const handleClose = () => setExecuteState(false);

        client.on("connect", handleConnect);
        client.on("error", handleError);
        client.on("close", handleClose);
        return () => {
            client.off("connect", handleConnect);
            client.off("error", handleError);
            client.off("close", handleClose);
        };
    }, []);

    return (
        <ConfigProvider locale={zhCN}>
            <div data-tauri-drag-region id="drag" style={{ position: "fixed", top: 0, height: "3vh", width: "100vw" }}>
                <Badge
                    className="title-badge"
                    color={executeState ? "hsl(102, 53%, 61%)" : "hsl(356, 77.46%, 54.78%)"}
                />
            </div>
            <div className="body">
                <Suspense fallback={<Loading />}>
                    <RouterProvider router={router} fallbackElement={<Loading />} />
                </Suspense>
            </div>
        </ConfigProvider>
    );
};

export default App;
