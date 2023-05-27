import Icon from "@ant-design/icons";

const NetWorkError = () => {
    return (
        <div className="network-error">
        <div className="network-error__inner">
            <div className="network-error__icon">
            <Icon type="icon-wangluo" />
            </div>
            <div className="network-error__title">网络错误</div>
            <div className="network-error__desc">请检查您的网络连接是否正常</div>
        </div>
        </div>
    );
};

export default NetWorkError;
