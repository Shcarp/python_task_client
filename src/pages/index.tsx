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

    return (
        <div>
            <Button onClick={() => handleClick()}>请求</Button>
        </div>
    );
};

export default Index;
