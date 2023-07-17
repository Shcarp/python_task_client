// import lottie from "lottie-web";
import classNames from "classnames";
import { useNavigate, useOutlet } from "react-router-dom";
import { Ref, createRef, useEffect, useRef, useState } from "react";
import { CSSTransition, SwitchTransition } from "react-transition-group";
import { IconFont } from "../../components/Iconfont";
// @ts-ignore
import { ReactComponent as ChatLog } from "@assets/svg/chat.svg";
import Background from "@assets/background.png";
import styles from "./index.module.less";
import "./animation.less";
import getStore from "../../utils/store";
import { AUTH_TOKEN } from "../../config";

export const UserCenter = () => {
    const navigate = useNavigate();
    const currentOutlet = useOutlet();
    const nodeRef = useRef<Record<string, Ref<HTMLDivElement>>>({
        "/user-center/login": createRef(),
        "/user-center/register": createRef(),
    });
    const [hide, setHide] = useState(true);

    useEffect(() => {
        (
            async () => {
                const store = await getStore();
                if (await store.get(AUTH_TOKEN)) {
                    // navigate("/");
                }
            }
        )()
    }, [])

    useEffect(() => {
        const verification = () => {
            if (location.pathname === "/user-center/login") {
                setHide(true);
            } else {
                setHide(false);
            }
        };
        verification();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [location.pathname]);

    return (
        <>
            <div className={styles["login-background"]}>
                <img className={classNames(styles["login-background-img"], styles["img1"])} src={Background} />
                <img className={classNames(styles["login-background-img"], styles["img2"])} src={Background} />
            </div>
            <div className={styles["login"]}>
                <div className={styles["animation"]}>
                    <IconFont type="icon-duanxinqunfa"></IconFont>
                    <span>PythonTask</span>
                </div>
                <div hidden={hide} className={styles["back"]}>
                    <IconFont
                        onClick={() => {
                            navigate("/user-center/login");
                        }}
                        type="wx_message-xiangyou"
                    ></IconFont>
                </div>
                <div className={styles["login-right"]}>
                    <ChatLog className={styles["login-chat"]} />
                </div>
                <div className={styles["login-left"]}>
                    <SwitchTransition mode="out-in">
                        <CSSTransition
                            addEndListener={() => {
                                // console.log(1);
                            }}
                            nodeRef={nodeRef.current[location.pathname]}
                            key={location.pathname}
                            classNames="fade"
                            unmountOnExit
                        >
                            {() => (
                                <div className={styles["content"]} ref={nodeRef.current[location.pathname]}>
                                    {currentOutlet}
                                </div>
                            )}
                        </CSSTransition>
                    </SwitchTransition>
                </div>
            </div>
        </>
    );
};

export default UserCenter;
