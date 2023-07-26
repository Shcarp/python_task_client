import { client } from "../../utils/client/websocket";
import { Left } from "./components/Left";
import { Right } from "./components/Right";

import styles from  "./index.module.less";

client.connect();

const Home = () => {
    return (
        <div className={styles.home}>
            <Left></Left>
            <Right></Right>
        </div>
    );
}

export default Home;
