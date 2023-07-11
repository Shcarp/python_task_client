import React, { Suspense, useEffect } from "react";
import { ConfigProvider } from "antd";
import { RouterProvider } from "react-router-dom";
import { Loading } from "./components/Loading";
import { router } from "./router";
import "dayjs/locale/zh-cn";
import zhCN from "antd/locale/zh_CN";
import "antd/dist/reset.css";
import "./App.less";

const App = () => {
    return (
        <ConfigProvider locale={zhCN}>
            <div className="body">
                <Suspense fallback={<Loading />}>
                    <RouterProvider router={router} fallbackElement={<Loading />} />
                </Suspense>
            </div>
        </ConfigProvider>
    );
};

export default App;
