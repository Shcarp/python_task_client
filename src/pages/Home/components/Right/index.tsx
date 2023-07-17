import { Outlet } from "react-router-dom";
import styles from "./index.module.less";

export const Right: React.FC = () => {
    return (
        <div className={styles.homeRight}>
            <Outlet></Outlet>
        </div>
    );
};
