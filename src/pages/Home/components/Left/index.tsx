// @ts-ignore
import { ReactComponent as TitleSvg } from "@assets/svg/title.svg";
import styles from "./index.module.less";
import { Avatar, Menu, MenuProps, Popover, Space, message } from "antd";
import { menu } from "../../../../router";
import { IconFont } from "../../../../components/Iconfont";
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { LogoutOutlined, UserOutlined } from "@ant-design/icons";
import getStore from "../../../../utils/store";
import { UserInfo } from "../../../../api/user-center.type";
import { AUTH_TOKEN } from "../../../../config";
import { sendLogout } from "../../../../api/user-center";

export const Left: React.FC = () => {
    const navigate = useNavigate();
    const [userInfo, setUserInfo] = useState<UserInfo | null>(null);
    const menuProps: MenuProps["items"] = menu.map((item) => {
        return {
            key: item.path,
            path: item.path,
            label: item.title,
            icon: <IconFont type={item.icon!}></IconFont>,
        };
    });

    const UserContentOptionsItem: MenuProps['items']= [
        {
            key: "logout",
            label: "退出登录",
            onClick: async () => {
                try {
                    await sendLogout()
                    const store = await getStore();
                    await store.delete("userInfo");
                    await store.delete(AUTH_TOKEN);
                    navigate("/user-center/login");
                } catch (error) {
                    console.log(error)
                }
            },
            icon: <LogoutOutlined />
        }
    ]

    const UserContent = (
        <div className={styles['userInfo-content']}>
            <Menu mode="inline" items={UserContentOptionsItem}></Menu>
        </div>
    );

    useEffect(() => {
        (async () => {
            const store = await getStore();
            const userData = await store.get<UserInfo>("userInfo");
            setUserInfo(userData);
        })();
    }, []);


    return (
        <div className={styles.homeLeft}>
            <div className={styles.title}>
                <TitleSvg height={200} width={200} classNames={styles.svg}></TitleSvg>
            </div>
            <Menu
                className={styles.leftMenu}
                onClick={(menuItem) => navigate(menuItem.key)}
                mode="inline"
                items={menuProps}
            ></Menu>
            <Popover className={styles.avatar} trigger="hover" content={UserContent}>
                <Space size={8}>
                    <Avatar shape="square" size={48} icon={<UserOutlined />} />
                    <div className={styles.name}>{userInfo?.username}</div>
                </Space>
            </Popover>
        </div>
    );
};
