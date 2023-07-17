import { appDataDir, join, dataDir, BaseDirectory, resolve } from "@tauri-apps/api/path";
import { Store } from "tauri-plugin-store-api";
import { exists } from '@tauri-apps/api/fs'
import { createFile } from "./utils";

export const getStore = async () => {
    const path = await resolve(await appDataDir(), "setting.dat");
    try {
        const isExists = await exists("setting.dat", {dir: BaseDirectory.AppData});
        if (!isExists){
            await createFile(path);
        }
        const store = new Store(path);
        return store;
    } catch (error) {
        throw new Error("获取store失败");
    }
}


export default getStore;
