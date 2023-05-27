import { invoke } from '@tauri-apps/api/tauri'

export const generate_messageid= () => {
    return invoke('generate_unique_message_id') as Promise<string> 
}