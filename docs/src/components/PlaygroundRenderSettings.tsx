import styles from '../pages/playground.module.css';

function PlaygroundRenderSettings({ onSubmit }: { onSubmit: () => Promise<void> }): JSX.Element {
  return (
    <div className={styles.settings}>
      <div className="row">
        <div className="col">Settings:</div>
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
