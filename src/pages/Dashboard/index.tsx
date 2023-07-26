import { Card, List, Image, Avatar, Form, Input, Popover, Select, Drawer, Alert, Button, Space, Row, Col } from "antd";
import { useEffect, useState } from "react";
import { GetScriptListParams, ScriptItem } from "../../api/script.type";
import { getScriptList, likeScript, favoriteScript, cancelFavoriteScript, cancelLikeScript } from "../../api/script";
import { EllipsisOutlined, FilterOutlined, SearchOutlined } from "@ant-design/icons";

import styles from "./index.module.less";
import { debounce } from "lodash";
import { DefaultOptionType } from "antd/es/select";
import { getScriptPlatforms } from "../../api/common";
import { IconFont } from "../../components/Iconfont";
import { ResponseBase } from "../../api/lib/type";
import { ScriptInfo } from "./ScriptInfo";

const { useForm } = Form;

const { Meta } = Card;

export const Dashboard: React.FC = () => {
    const [searchForm] = useForm();
    const [filterForm] = useForm();
    const [open, setOpen] = useState(false);
    const [platformList, setPlatformList] = useState<DefaultOptionType[]>([]);
    const [listParams, setListParams] = useState<GetScriptListParams>({});
    const [dataSource, setDataSource] = useState<ScriptItem[]>();

    const [currentSelect, setCurrentSelect] = useState<ScriptItem>();

    const query = async () => {
        const res = await getScriptList(listParams);
        setDataSource(res.data);
        // 找到当前选中的脚本
        const current = res.data.find((item) => item.scriptUid === currentSelect?.scriptUid);
        if (current) {
            setCurrentSelect(current);
        }
    };

    const handleChange = debounce((value: any, values: any) => {
        setListParams({
            ...listParams,
            ...values,
            scriptPlatformId: values.scriptPlatformId?.value ?? 3,
        });
    }, 500);

    const requestMap: Record<string, (scriptUid: string) => Promise<ResponseBase>> = {
        like: likeScript,
        unlike: cancelLikeScript,
        unfavorite: cancelFavoriteScript,
        favorite: favoriteScript,
    };

    const handleActionsClick = (e: React.MouseEvent<HTMLDivElement, MouseEvent>, key: string, id: string) => {
        e.stopPropagation();
        const request = requestMap[key];
        if (request) {
            request(id)
                .then((res) => {
                    query();
                })
                .catch((err) => {
                    console.log(err);
                });
        }
    };

    useEffect(() => {
        query();
    }, [listParams]);

    useEffect(() => {
        (async () => {
            const res = await getScriptPlatforms();
            setPlatformList(
                res.data.map((item) => ({
                    label: item.platformName,
                    value: item.platformId,
                }))
            );
        })();
    }, []);

    const FilterContent = (
        <div className={styles.filterContent}>
            <Form layout="vertical" form={filterForm} onValuesChange={handleChange}>
                <Form.Item name="scriptPlatformId" label="选择平台">
                    <Select allowClear options={platformList} placeholder="请输入脚本名称"></Select>
                </Form.Item>
            </Form>
        </div>
    );

    return (
        <>
            <div className={styles.header}>
                <Form layout="vertical" form={searchForm} onValuesChange={handleChange}>
                    <Form.Item name="keyword">
                        <Input
                            prefix={<SearchOutlined />}
                            suffix={
                                <Popover content={FilterContent} trigger="hover">
                                    过滤
                                    <FilterOutlined />
                                </Popover>
                            }
                            placeholder="搜索"
                        ></Input>
                    </Form.Item>
                </Form>
            </div>
            <div className={styles.content} id="scrollableDiv">
                <List
                    grid={{
                        gutter: 16,
                        column: 4,
                    }}
                    dataSource={dataSource}
                    rowKey="scriptUid"
                    renderItem={(item: ScriptItem) => (
                        <>
                            {item ? (
                                <List.Item>
                                    <Card
                                        onClick={() => {
                                            setCurrentSelect(item);
                                            setOpen(true);
                                        }}
                                        hoverable={true}
                                        cover={
                                            <Image
                                                alt="example"
                                                src="https://gw.alipayobjects.com/zos/rmsportal/JiqGstEfoWAOHiTxclqi.png"
                                            />
                                        }
                                        actions={[
                                            item.isLike ? (
                                                <div
                                                    key="unlike"
                                                    onClick={(e) => handleActionsClick(e, "unlike", item.scriptUid)}
                                                >
                                                    <IconFont type="wx_message-shoucang2" />
                                                </div>
                                            ) : (
                                                <div
                                                    key="like"
                                                    onClick={(e) => handleActionsClick(e, "like", item.scriptUid)}
                                                >
                                                    <IconFont type="wx_message-xihuan" />
                                                </div>
                                            ),
                                            item.isFavorite ? (
                                                <div
                                                    key="unfavorite"
                                                    onClick={(e) => handleActionsClick(e, "unfavorite", item.scriptUid)}
                                                >
                                                    <IconFont type="wx_message-shoucang1" />
                                                </div>
                                            ) : (
                                                <div
                                                    key="favorite"
                                                    onClick={(e) => handleActionsClick(e, "favorite", item.scriptUid)}
                                                >
                                                    <IconFont type="wx_message-shoucang" />
                                                </div>
                                            ),
                                            <div key="ellipsis" data-key="like" data-id={item.scriptUid}>
                                                <EllipsisOutlined />,
                                            </div>,
                                        ]}
                                    >
                                        <>
                                            <div className={styles.likeAndF}>
                                                <span className={styles.likeAndFItem}>
                                                    <IconFont
                                                        className={styles.icon}
                                                        type="wx_message-xihuan"
                                                    ></IconFont>
                                                    {item.likes}
                                                </span>
                                                <span className={styles.likeAndFItem}>
                                                    <IconFont
                                                        className={styles.icon}
                                                        type="wx_message-shoucang"
                                                    ></IconFont>
                                                    {item.favorites}
                                                </span>
                                            </div>
                                            <Meta
                                                className={styles["card-meta"]}
                                                avatar={
                                                    <Avatar src="https://xsgames.co/randomusers/avatar.php?g=pixel" />
                                                }
                                                title={item.scriptName}
                                                description={item.scriptDescription}
                                            />
                                        </>
                                    </Card>
                                </List.Item>
                            ) : null}
                        </>
                    )}
                ></List>
            </div>
            <Drawer
                title={
                    <div className={styles.drwTitle}>
                        <Space>
                            <Avatar
                                src={currentSelect?.avatar ?? "https://xsgames.co/randomusers/avatar.php?g=pixel"}
                            />
                        </Space>
                        <Space>
                            <Button
                                onClick={(e: any) =>
                                    handleActionsClick(
                                        e,
                                        currentSelect?.isFavorite ? "unfavorite" : "favorite",
                                        currentSelect?.scriptUid ?? ""
                                    )
                                }
                                icon={
                                    currentSelect?.isFavorite ? (
                                        <IconFont type="wx_message-shoucang1" />
                                    ) : (
                                        <IconFont type="wx_message-shoucang" />
                                    )
                                }
                            ></Button>
                            <Button
                                onClick={(e: any) =>
                                    handleActionsClick(
                                        e,
                                        currentSelect?.isLike ? "unlike" : "like",
                                        currentSelect?.scriptUid ?? ""
                                    )
                                }
                                icon={
                                    currentSelect?.isLike ? (
                                        <IconFont type="wx_message-shoucang2" />
                                    ) : (
                                        <IconFont type="wx_message-xihuan" />
                                    )
                                }
                            ></Button>
                            <Button type="primary">使用</Button>
                        </Space>
                    </div>
                }
                placement="right"
                mask={false}
                onClose={() => setOpen(false)}
                open={open}
            >
                <ScriptInfo scriptUid={currentSelect?.scriptUid}></ScriptInfo>
            </Drawer>
        </>
    );
};

export default Dashboard;
