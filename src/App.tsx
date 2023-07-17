import { window as tauriWindow } from "@tauri-apps/api";
import React, { Suspense, useEffect } from "react";
import { RouterProvider } from "react-router-dom";
import { Loading } from "./components/Loading";
import { router } from "./router";
import "dayjs/locale/zh-cn";
import zhCN from "antd/locale/zh_CN";
import "antd/dist/reset.css";
import "./App.less";
import { Badge, ConfigProvider } from "antd";

const App = () => {
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
    return (
        <ConfigProvider locale={zhCN}>
            <div data-tauri-drag-region id="drag" style={{ position: "fixed", top: 0, height: "3vh", width: "100vw" }}>
                <Badge className="title-badge" color="hsl(102, 53%, 61%)" text="connected" />
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
