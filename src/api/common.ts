import { PlatformItem } from "./common.type";
import http from "./lib";
import { ResponseData } from "./lib/type";


export async function getScriptPlatforms(){
    const res = await http.get<ResponseData<PlatformItem[]>>("/v1/common/platforms");
    return res;
}


