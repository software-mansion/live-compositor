import clsx from 'clsx';
import styles from '../pages/playground.module.css';

function PlaygroundRenderSettings({ onSubmit }) {
  return (
    <div className={clsx(styles.settings)}>
      <div className="row">
        <div className="col">Resolution:</div>
        <div className="col">
          <select>
            <option value="someOption">Some option</option>
            <option value="otherOption">Other option</option>
          </select>
        </div>
        <div className="col">
          <select>
            <option value="someOption">Some option</option>
            <option value="otherOption">Other option</option>
          </select>
        </div>
        <div className="col">
          <select>
            <option value="someOption">Some option</option>
            <option value="otherOption">Other option</option>
          </select>
        </div>
        <div className="col">
          <button className="button button--outline button--primary" onClick={onSubmit}>
            Submit
          </button>
        </div>
      </div>
    </div>
  );
}

export default PlaygroundRenderSettings;
