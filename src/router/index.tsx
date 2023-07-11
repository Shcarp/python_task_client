
import React from "react";
import { lazy } from "react";
import { createBrowserRouter } from "react-router-dom";

const Index = lazy(() => import("../pages/index"));

export const routerMap = [
    {
        path: "/",
        element: <Index />,
    }
]

export const router = createBrowserRouter(routerMap, {
    future: {
        // Normalize `useNavigation()`/`useFetcher()` `formMethod` to uppercase
        v7_normalizeFormMethod: true,
      },
});
