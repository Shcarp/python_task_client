import { Store } from "tauri-plugin-store-api";

const getStore = async () => {
    const store = new Store(`./data/task.dat`);
    return store
}

export default getStore;
