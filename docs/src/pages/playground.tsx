import clsx from 'clsx';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './playground.module.css';
import { useState } from 'react';
import Options from '../components/options';
import ImageDisplay from '../components/imageDisplay';
import CodeInputArea from '../components/codeInputArea';
import toast, { Toaster } from 'react-hot-toast';

const backendUrl = 'http://localhost:8081/render_image';

function Homepage() {
  const [textAreaValue, setTextAreaValue] = useState('');
  const [imageUrl, setImageUrl] = useState('');

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
        body: textAreaValue,
      });

      if (response.status >= 400) {
        const contentType = response.headers.get('Content-Type');
        const errorStatus = `Error status: ${response.status}`;
        let errorMessage = '';

        if (contentType === 'text/plain; charset=utf-8') {
          const txt = await response.text();
          errorMessage = `Error message:\n${txt}`;
        } else if (contentType === 'application/json') {
          const apiError = await response.json();
          errorMessage = `Error message:\n${apiError.stack}`;
        }

        throw new Error(`${errorStatus}\n${errorMessage}`);
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
        <div className={clsx(styles.codeArea)}>
          <CodeInputArea onChange={handleInputChange} />
        </div>
      </div>
      <div className={clsx(styles.rightSide)}>
        <div className={clsx(styles.imageDisplay)}>
          <ImageDisplay imageUrl={imageUrl} />
        </div>
        <div className={clsx(styles.options)}>
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
