import clsx from 'clsx';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './playground.module.css';
import { useState } from 'react';

const CodeInputArea = ({ value, onChange }) => (
  <textarea
    className={clsx(styles.codeArea)}
    name="inputArea"
    placeholder="Enter your code to try it out"
    onChange={onChange}
    style={{ height: value, width: '100%' }}
  />
);

const ImageDisplay = ({ imageUrl }) => (
  <div className={clsx(styles.imageDisplay)}>
    {imageUrl ? (
      <img
        src={imageUrl}
        alt="Generated"
        style={{
          objectFit: 'contain',
          height: '100%',
          width: '100%',
        }}
      />
    ) : (
      <div></div>
    )}
  </div>
);

const Options = ({ onSubmit }) => (
  <div className={clsx('row', styles.options)}>
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
);

function Homepage() {
  const [textAreaValue, setTextAreaValue] = useState('');
  const [imageUrl, setImageUrl] = useState('');

  const handleInputChange = event => {
    setTextAreaValue(event.target.value);
  };

  const handleSubmit = async () => {
    const backendUrl = 'http://localhost:8081/render_image';

    try {
      const response = await fetch(backendUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: textAreaValue,
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const blob = await response.blob();
      const imageObjectURL = URL.createObjectURL(blob);
      setImageUrl(imageObjectURL);
    } catch (error) {
      console.error('Error:', error);
    }
  };

  return (
    <div className={clsx(styles.page)}>
      <div className="row">
        <div className={clsx('col col-6', styles.codeArea)}>
          <CodeInputArea value={textAreaValue} onChange={handleInputChange} />
        </div>
        <div className="col col-6">
          <ImageDisplay imageUrl={imageUrl} />
          <Options onSubmit={handleSubmit} />
        </div>
      </div>
    </div>
  );
}

export default function Home(): JSX.Element {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title}>
      <Homepage />
    </Layout>
  );
}
