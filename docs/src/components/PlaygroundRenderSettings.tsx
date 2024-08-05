import { Tooltip } from 'react-tooltip';
import 'react-tooltip/dist/react-tooltip.css';

function PlaygroundRenderSettings({
  onSubmit,
  active,
}: {
  onSubmit: () => Promise<void>;
  active: boolean;
}) {
  return (
    <div style={{ margin: '10px' }}>
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
        <div
          className="col"
          data-tooltip-id={active ? null : 'disableSubmit'}
          data-tooltip-content={active ? null : 'Enter valid JSON!'}
          data-tooltip-place={active ? null : 'top'}>
          <button
            className={`button ${
              active ? 'button--outline button--primary' : 'disabled button--secondary'
            }`}
            style={active ? {} : { color: '#f5f5f5', backgroundColor: '#dbdbdb' }}
            onClick={onSubmit}>
            Submit
          </button>
        </div>
        <Tooltip id="disableSubmit" />
      </div>
    </div>
  );
}

export default PlaygroundRenderSettings;
