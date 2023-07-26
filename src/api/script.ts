import http from "./lib";
import { ResponseBase, ResponseData } from "./lib/type";
import { CreateScriptParams, GetScriptListParams, ScriptDetail, ScriptItem, UserScriptListItem } from "./script.type";

export async function getScriptList(params: GetScriptListParams) {
    const res = await http.get<ResponseData<ScriptItem[]>>("/v1/script/list", {
        params: params,
    });
    return res;
}

export async function getScriptDetail(id: string) {
    const res = await http.get<ResponseData<ScriptDetail>>(`/v1/script/detail`, {
        params: {
            scriptUid: id,
        },
    });
    return res;
}

export async function createScript(body: CreateScriptParams) {
    const res = await http.post<ResponseBase>("/v1/script/create", body);
    return res;
}

export async function updateScript(body: CreateScriptParams & { scriptUid: string }) {
    const res = await http.post<ResponseBase>("/v1/script/update", body);
    return res;
}

/**
 * 点赞
 */
export async function likeScript(scriptUid: string) {
    const res = await http.get<ResponseBase>("/v1/script/like", {
        params: {
            scriptUid,
        }
    });
    return res;
}

/**
 * 取消点赞
 */
export async function cancelLikeScript(scriptUid: string) {
    const res = await http.get<ResponseBase>("/v1/script/unlike", {
        params: {
            scriptUid,
        }
    });
    return res;
}

/**
 * 收藏
 */
export async function favoriteScript(scriptUid: string) {
    const res = await http.get<ResponseBase>("/v1/script/favorite", {
        params: {
            scriptUid,
        }
    });
    return res;
}

/**
 * 取消收藏
 */
export async function cancelFavoriteScript(scriptUid: string) {
    const res = await http.get<ResponseBase>("/v1/script/unfavorite", {
        params: {
            scriptUid,
        }
    });
    return res;
}

/**
 * 获取用户脚本
 */
export async function getUserScriptList(params: GetScriptListParams) {
    const res = await http.get<ResponseData<UserScriptListItem[]>>("/v1/script/list/user", {
        params: params,
    });
    return res;
}

/**
 * 获取点赞脚本
 */

export async function getLikeScriptList(params: GetScriptListParams) {
    const res = await http.get<ResponseData<UserScriptListItem[]>>("/v1/script/list/user/like", {
        params: params,
    });
    return res;
}

/**
 * 获取收藏脚本
*/
export async function getFavoriteScriptList(params: GetScriptListParams) {
    const res = await http.get<ResponseData<UserScriptListItem[]>>("/v1/script/list/user/favorite", {
        params: params,
    });
    return res;
}
