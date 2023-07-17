import { Button } from "antd";
import http from "../api/lib"

import { ResponseType } from "@tauri-apps/api/http";

const Index = () => {
    const handleClick = async () => {
        const res = await http.get("/v1/script/list", {
            params: {
                current: 1,
                pageSize: 10
            },
            responseType: ResponseType.JSON
        })
        console.log(res);
    };

    const handleLogin = async () => {
        window.location.href = "/user-center/login";
    }

    return (
        <div>
            <Button onClick={() => handleClick()}>请求</Button>
            <Button onClick={() => handleLogin()}>Open Login</Button>
        </div>
    );
};

export default Index;
