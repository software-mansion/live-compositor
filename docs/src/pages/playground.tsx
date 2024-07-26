import clsx from 'clsx';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './playground.module.css';
import { useState } from 'react';
import Options from '../components/PlaygroundRenderSettings';
import ImageDisplay from '../components/PlaygroundPreview';
import CodeInputArea from '../components/PlaygroundCodeEditor';
import toast, { Toaster } from 'react-hot-toast';

const backendUrl = 'http://localhost:8081/render_image';

function Homepage() {
  const [textAreaValue, setTextAreaValue] = useState(
    JSON.stringify(
      {
        type: 'view',
      },
      null,
      2
    )
  );
  const [imageUrl, setImageUrl] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  const handleInputChange = event => {
    setTextAreaValue(event.target.value);
  };

  const handleSubmit = async () => {
    try {
      const response = await fetch(backendUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: `{ "scene": ${textAreaValue} }`,
      });

      if (response.status >= 400) {
        const contentType = response.headers.get('Content-Type');
        const errorStatus = `Error status: ${response.status}`;
        let errorMsg = '';

        if (contentType === 'application/json') {
          const apiError = await response.json();
          if (apiError.stack) {
            errorMsg = `Error message: ${apiError.stack.map(String).join('\n')}`;
          } else {
            errorMsg = `Error message: ${apiError.message}`;
          }
        } else {
          const txt = await response.text();
          errorMsg = `Error message: ${txt}`;
        }

        setErrorMessage(errorMsg);
        throw new Error(`${errorStatus}\n${errorMsg}`);
      } else {
        setErrorMessage('');
      }

      const blob = await response.blob();
      const imageObjectURL = URL.createObjectURL(blob);
      setImageUrl(imageObjectURL);
    } catch (error) {
      toast.error(`${error.message}`);
    }
  };

  return (
    <div className={clsx(styles.page)}>
      <div className={clsx(styles.leftSide)}>
        <div className={clsx(styles.codeEditor)}>
          <CodeInputArea textAreaValue={textAreaValue} onChange={handleInputChange} />
        </div>
      </div>
      <div className={clsx(styles.rightSide)}>
        <div className={clsx(styles.preview)}>
          <ImageDisplay imageUrl={imageUrl} errorMessage={errorMessage} />
        </div>
        <div className={clsx(styles.settingsBox)}>
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
      <Toaster />
      <Homepage />
    </Layout>
  );
}
