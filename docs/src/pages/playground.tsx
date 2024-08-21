import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './playground.module.css';
import React, { useEffect, useState } from 'react';
import PlaygroundRenderSettings from '../components/PlaygroundRenderSettings';
import PlaygroundPreview from '../components/PlaygroundPreview';
import PlaygroundCodeEditor from '../components/PlaygroundCodeEditor';
import toast, { Toaster } from 'react-hot-toast';
import { ApiError, renderImage } from '../api';
import 'react-tooltip/dist/react-tooltip.css';
import PlaygroundReactEditor from '../components/PlaygroundReactEditor';
import playgroundReactRunner from '../playgroundReactRunner';

const INITIAL_SCENE = {
  type: 'view',
};

const INITIAL_REACT_CODE = [
  "import React from 'react';\n",

  'function View() {',
  '    return null;',
  '}',
  'function a(): JSX.Element {',
  '    return (',
  '        <div>',
  '            <View/>',
  '        </div>',
  '    )',
  '}',
  'console.log(a());',
  'console.log("Hello");',
].join('\n');

function Homepage() {
  const [scene, setScene] = useState<object | Error>(INITIAL_SCENE);
  const [code, setCode] = useState<string>(INITIAL_REACT_CODE);
  const [showReactEditor, setShowReactEditor] = useState<boolean>(false);

  const [responseData, setResponseData] = useState({
    imageUrl: '',
    errorMessage: '',
  });

  const setErrorMessage = message => {
    setResponseData(prevResponseData => ({ ...prevResponseData, errorMessage: message }));
  };

  const handleSubmit = async (): Promise<void> => {
    try {
      if (showReactEditor) {
        await playgroundReactRunner(code);
      } else {
        if (scene instanceof Error) {
          throw new Error(`${scene.name};\n${scene.message}`);
        }
        const blob = await renderImage({ scene });
        const imageObjectURL = URL.createObjectURL(blob);

        setResponseData({ imageUrl: imageObjectURL, errorMessage: '' });
      }
    } catch (error: any) {
      let errorDescription;
      if (error instanceof ApiError && !error.response) {
        errorDescription = 'Failed to connect to the server!';
      } else {
        errorDescription = error.message;
      }
      setErrorMessage(errorDescription);
      toast.error(`${errorDescription}`);
    }
  };

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const codeEditorMode = params.get('mode');
    if (codeEditorMode === 'react') {
      setShowReactEditor(true);
    } else {
      setShowReactEditor(false);
    }
  }, []);

  return (
    <div className={styles.page}>
      <div className={styles.leftSide}>
        <div className={styles.codeEditorBox}>
          {showReactEditor ? (
            <PlaygroundReactEditor code={code} onCodeChange={setCode} />
          ) : (
            <PlaygroundCodeEditor onChange={setScene} initialCodeEditorContent={INITIAL_SCENE} />
          )}
        </div>
      </div>
      <div className={styles.rightSide}>
        <div className={styles.preview}>
          <PlaygroundPreview {...responseData} />
        </div>
        <div className={styles.settingsBox}>
          <PlaygroundRenderSettings
            onSubmit={handleSubmit}
            readyToSubmit={!(scene instanceof Error) || showReactEditor}
          />
        </div>
      </div>
    </div>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title}>
      <Toaster />
      <Homepage />
    </Layout>
  );
}
