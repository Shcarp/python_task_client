import { lazy } from "react";
import { createBrowserRouter } from "react-router-dom";

const Index = lazy(() => import("../pages/index"));
const Home = lazy(() => import("../pages/Home"));
const UserCenter = lazy(() => import("../pages/user-center"));
const Login = lazy(() => import("../pages/user-center/Login"));
const Register = lazy(() => import("../pages/user-center/Register"));
const Setting = lazy(() => import("../pages/Setting"));
const User = lazy(() => import("../pages/User"));
const Dashboard = lazy(() => import("../pages/Dashboard"));

interface RouterItem {
    path: string;
    element: JSX.Element;
    children?: RouterItem[];
    title?: string;
    isMenu?: boolean;
    icon?: string;
}

export const routerMap: RouterItem[] = [
    {
        path: "/",
        element: <Home />,
        children: [
            {
                path: "/",
                element: <Dashboard />,
                title: "仪表盘",
                isMenu: true,
                icon: "wx_message-dashboard",
            },
            {
                path: "user",
                element: <User />,
                title: "用户",  
                isMenu: true,
                icon: "wx_message-user",
            },
            {
                path: "setting",
                element: <Setting />,
                title: "系统设置",
                isMenu: true,
                icon: "wx_message-setting",
            },
        ],
    },
    {
        path: "/user-center",
        element: <UserCenter />,
        children: [
            {
                path: "login",
                element: <Login />,
            },
            {
                path: "register",
                element: <Register />,
            },
        ],
    },
    {
        path: "/test",
        element: <Index />,
    },
];

export const router = createBrowserRouter(routerMap, {
    future: {
        // Normalize `useNavigation()`/`useFetcher()` `formMethod` to uppercase
        v7_normalizeFormMethod: true,
    },
});

const res: RouterItem[] = [];
const dealRouter = (cur: RouterItem[], root: string) => {
    cur.forEach((item) => {
        const path = root === "" 
                        ? item.path 
                        : item.path.startsWith("/")
                        ? root === "/" 
                        ? item.path 
                        : root + item.path
                        : root.endsWith("/") 
                        ? root + item.path 
                        : root + "/" + item.path;

        if (item.children) {
            dealRouter(item.children, path);
        } else {
            if (item.isMenu) {
                res.push({
                    path: path,
                    title: item.title ?? "",
                    element: item.element,
                    icon: item.icon ?? "",
                });
            }
        }
    });
};
dealRouter(routerMap, "")

export const menu = res;

