export enum ScriptVisibilityStatus {
    PUBLIC = 1,
    PRIVATE = 0,
}

export interface GetScriptListParams {
    // current: number;
    // pageSize: number;
    keyword?: string;
    scriptModule?: string;
    scriptUid?: string;
    scriptPlatformId?: string;
}

export interface ScriptItem {
    scriptId: number;
    scriptUid: string;
    scriptName: string;
    scriptModule: string;
    scriptKey: string;
    scriptVersion: string;
    scriptDescription: string;
    scriptDetailedDescription: string;
    scriptConfigText: string;
    scriptPlatformId: number;
    scriptVisibility: ScriptVisibilityStatus;
    likes: number;
    isFavorite: 0 | 1;
    isLike: 0 | 1;
    favorites: number;
    created_at: string;
    updated_at: string;
    avatar: string;
    scriptIcon: string;
}

export type ScriptListItem = Omit<
    ScriptItem,
    | "scriptUid"
    | "scriptDescription"
    | "scriptDetailedDescription"
    | "scriptVisibility"
    | "likes"
    | "favorites"
    | "isFavorite"
    | "isLike"
>;

export interface ScriptDetail {
    scriptUid: string;
    scriptName: string;
    scriptDescription: string;
    scriptDetailedDescription: string;
    scriptVisibility: number;
    likes: number;
    favorites: number;
    isFavorite: number;
    isLike: number;
    items: ScriptListItem[];
}

export interface CreateScriptParams {
    scriptVersion: string;
    scriptName: string;
    scriptModule: string;
    scriptKey: string;
    scriptDescription: string;
    scriptDetailedDescription: string;
    scriptConfigText: string;
    scriptPlatformId: 3;
    scriptVisibility: ScriptVisibilityStatus;
}

export type UserScriptListItem = Omit<ScriptItem, "isFavorite" | "isLike">
