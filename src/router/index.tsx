
import React from "react";
import { lazy } from "react";
import { NonIndexRouteObject, createBrowserRouter } from "react-router-dom";

const Test = lazy(() => import("../pages/Test"));
const Home = lazy(() => import("../pages/Home"));
const NotFound = lazy(() => import("../components/NotFound"));
const Task = lazy(() => import("../pages/Task"));
const Connect = lazy(() => import("../pages/Connect"));

export interface RouterMapItem extends NonIndexRouteObject {
    tool_id?: string;
    description?: string;
    isTool?: boolean;
    tool_name?: string;
    tool_image?: string;
}

export const routerMap: RouterMapItem[] = [
    {
        path: "/",
        element: <Test />,
    },
    {
        path: "/task",
        element: <Task />,
    },
    {
        path: "/connect",
        element: <Connect />,
    },
    {
        path: "*",
        element: <NotFound />,
    }
]

export const router = createBrowserRouter(routerMap, {
    future: {
        // Normalize `useNavigation()`/`useFetcher()` `formMethod` to uppercase
        v7_normalizeFormMethod: true,
      },
});
