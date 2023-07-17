import { Left } from "./components/Left";
import { Right } from "./components/Right";

import styles from  "./index.module.less";

const Home = () => {
    return (
        <div className={styles.home}>
            <Left></Left>
            <Right></Right>
        </div>
    );
}

export default Home;
