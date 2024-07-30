import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './playground.module.css';
import { useState } from 'react';
import PlaygroundRenderSettings from '../components/PlaygroundRenderSettings';
import PlaygroundPreview from '../components/PlaygroundPreview';
import PlaygroundCodeEditor from '../components/PlaygroundCodeEditor';
import toast, { Toaster } from 'react-hot-toast';
import { buildRequestSceneObject, handleRenderImageRequest } from '../api';

const BACKEND_URL: URL = new URL('http://localhost:8081');

function Homepage(): JSX.Element {
  const [scene, setScene] = useState<object | Error>({
    type: 'view',
  });

  const [responseData, setResponseData] = useState({
    imageUrl: '',
    errorMessage: '',
  });

  const setErrorMessage = message => {
    setResponseData(prevResponseData => ({ ...prevResponseData, errorMessage: message }));
  };

  const handleCodeEditorChange = (scene: object | Error): void => {
    setScene(scene);
  };

  const handleSubmit = async (): Promise<void> => {
    try {
      const requestObject = buildRequestSceneObject('POST', scene);

      const blob = await handleRenderImageRequest(BACKEND_URL, requestObject);
      const imageObjectURL = URL.createObjectURL(blob);

      setResponseData({ imageUrl: imageObjectURL, errorMessage: '' });
    } catch (error: any) {
      setErrorMessage(error.message);
      toast.error(`${error.message}`);
    }
  };

  return (
    <div className={styles.page}>
      <div className={styles.leftSide}>
        <div className={styles.codeEditor}>
          <PlaygroundCodeEditor
            onChange={handleCodeEditorChange}
            initialCodeEditorContent={JSON.stringify(scene, null, 2)}
          />
        </div>
      </div>
      <div className={styles.rightSide}>
        <div className={styles.preview}>
          <PlaygroundPreview responseData={responseData} />
        </div>
        <div className={styles.settingsBox}>
          <PlaygroundRenderSettings onSubmit={handleSubmit} />
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
