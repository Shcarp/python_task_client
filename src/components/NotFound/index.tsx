import React, { Result, Button } from 'antd';

const NotFound = () => {
    return (
        <div className="remote-monitor">
            <Result
                status="404"
                title="404"
                subTitle={"页面未找到"}
                extra={
                    <Button type="primary" href="/">
                        返回主页
                    </Button>
                }
            />
        </div>
    );
};

export default NotFound;
