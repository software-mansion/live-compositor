import clsx from 'clsx';
import styles from '../pages/playground.module.css';

function PlaygroundCodeEditor({ onChange, textAreaValue }) {
  return (
    <textarea
      className={clsx(styles.codeEditor)}
      name="inputArea"
      placeholder="Enter your code to try it out"
      onChange={onChange}
      value={textAreaValue}
    />
  );
}
export default PlaygroundCodeEditor;
