import { Outlet } from "react-router-dom";
import styles from "./index.module.less";
import { Suspense } from "react";
import { Loading } from "../../../../components/Loading";

export const Right: React.FC = () => {
    return (
        <div className={styles.homeRight}>
            <Suspense fallback={<Loading />}>
                <Outlet></Outlet>
            </Suspense>
        </div>
    );
};
