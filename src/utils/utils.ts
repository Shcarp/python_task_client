import { invoke } from '@tauri-apps/api/tauri'

export const createFile = async (path: string) => {
    await invoke('create_file', { path })
}
