const Options = ({ onSubmit }) => (
  <div className="bottom-right-component">
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

export default Options;
