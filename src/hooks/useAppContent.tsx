import { createContext, useContext } from "react";
import { State } from "../utils/client/websocket";

interface AppContext {
    client_state: State;
}

export const AppContext = createContext<AppContext>({
    client_state: State.CLOSED,
});

export function useAppContext() {
    const appContext = useContext(AppContext);
    return appContext;
}
