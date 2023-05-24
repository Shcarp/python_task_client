import React from "react";
import { ConfigProvider } from "antd";
import "dayjs/locale/zh-cn";
import zhCN from "antd/locale/zh_CN";
import "antd/dist/reset.css";
import "./index.css"

interface IselfProps {
    Component: React.FC;
    pageProps: any;
}

const App = ({ Component, pageProps }: IselfProps) => {
    return (
        <ConfigProvider locale={zhCN}>
            <Component {...pageProps} />
        </ConfigProvider>
    );
};

export default App;
