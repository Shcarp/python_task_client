import http from "./lib";
import { ResponseBase, ResponseData } from "./lib/type";
import { UserInfo } from "./user-center.type";


export async function sendLogin(username: string, password: string){
    const res = await http.post<ResponseData<UserInfo>>("/v1/login", {
        username,
        password,
    });
    return res;
}

export async function sendRegister(username: string, password: string){
    const res = await http.post<ResponseBase>("/v1/register", {
        username,
        password,
    });
    return res;
}

export async function sendLogout(){
    const res = await http.get<ResponseBase>("/v1/logout");
    return res;
}

export async function checkUsernameExist(name: string){
    const res = await http.get<ResponseData<boolean>>("/v1/checkUsernameExist", {
        params: {
            username: name,
        },
    });
    return res;
}
