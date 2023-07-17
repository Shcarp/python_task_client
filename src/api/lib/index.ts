import { HttpOptions } from "@tauri-apps/api/http";
import { http } from "../../utils/httpClient";
import { dialog } from "@tauri-apps/api";
import getStore from "../../utils/store";
import { message } from "antd";
import { AUTH_TOKEN } from "../../config";

http.baseUrl = "http://127.0.0.1:3306";

http.interceptorsRequest = async (config: HttpOptions) => {
    const store = await getStore();
    let token = "";
    if (await store.has(AUTH_TOKEN)) {
        token = await store.get(AUTH_TOKEN) ?? "";
    }
    config.headers = {
        Authorization: token,
    };
    return config;
};

http.interceptorsResponse = async (response) => {
    if (response.status === 401) {
        dialog
            .confirm("", {
                title: "登录过期，请重新登录",
                type: "warning",
            })
            .then((res) => {
                if (res === true) {
                    location.href = "/user-center/login";
                }
            });
    }

    if (response.status === 403) {
        message.warning("没有权限");
    }

    if (response.status === 500) {
        message.error("服务器错误");
    }

    if (response.data.code !== 0) {
        message.error(response.data.msg);
    }

    return response;
};

export default http;
