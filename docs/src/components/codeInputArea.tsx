import clsx from 'clsx';
import styles from '../pages/playground.module.css';

const CodeInputArea = ({ onChange }) => (
  <textarea
    className={clsx(styles.codeArea)}
    name="inputArea"
    placeholder="Enter your code to try it out"
    onChange={onChange}
  />
);
export default CodeInputArea;
