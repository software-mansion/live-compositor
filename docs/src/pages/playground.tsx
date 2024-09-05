import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import { useEffect, useState } from 'react';
import toast, { Toaster } from 'react-hot-toast';
import 'react-tooltip/dist/react-tooltip.css';
import { ApiError, renderImage } from '../api';
import PlaygroundCodeEditor from '../components/PlaygroundCodeEditor';
import PlaygroundPreview from '../components/PlaygroundPreview';
import PlaygroundReactEditor from '../components/PlaygroundReactEditor';
import PlaygroundSettings from '../components/PlaygroundSettings';
import executeTypescriptCode from '../executeTypescriptCode';
import {
  InputResolution,
  inputResolutionsToResolutions,
  InputsSettings,
  Resolution,
} from '../resolution';
import styles from './playground.module.css';

const INITIAL_SCENE = {
  type: 'view',
  background_color_rgba: '#4d4d4dff',
  children: [
    {
      type: 'rescaler',
      child: { type: 'input_stream', input_id: 'input_1' },
    },
    {
      type: 'rescaler',
      width: 320,
      height: 180,
      top: 20,
      right: 20,
      child: { type: 'input_stream', input_id: 'input_2' },
    },
    {
      type: 'rescaler',
      width: 320,
      height: 180,
      top: 20,
      left: 20,
      child: { type: 'input_stream', input_id: 'input_3' },
    },
    {
      type: 'rescaler',
      width: 320,
      height: 180,
      bottom: 20,
      left: 20,
      child: { type: 'input_stream', input_id: 'input_4' },
    },
    {
      type: 'rescaler',
      width: 320,
      height: 180,
      bottom: 20,
      right: 20,
      child: { type: 'input_stream', input_id: 'input_5' },
    },
    {
      type: 'rescaler',
      width: 640,
      height: 400,
      top: 20,
      right: 800,
      child: { type: 'input_stream', input_id: 'input_6' },
    },
  ],
};

const INITIAL_REACT_CODE = [
  "import React from 'react';\n",
  "import { View } from 'live-compositor';\n",
  'function a(): JSX.Element {',
  '    return (',
  '        <div>',
  '            <View direction="column" />',
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
  const [inputResolutions, setInputResolutions] = useState<InputsSettings>({
    input_1: InputResolution.Resoultion1920x1080,
    input_2: InputResolution.Resoultion1920x1080,
    input_3: InputResolution.Resoultion1920x1080,
    input_4: InputResolution.Resoultion1920x1080,
    input_5: InputResolution.Resoultion1920x1080,
    input_6: InputResolution.Resoultion1920x1080,
  });

  function updateInputResolutions(inputId: string, resolution: InputResolution) {
    setInputResolutions({
      ...inputResolutions,
      [inputId]: resolution,
    });
  }
  const [outputResolution, setOutputResolution] = useState<Resolution>({
    width: 1920,
    height: 1080,
  });

  const [responseData, setResponseData] = useState({
    imageUrl: '',
    errorMessage: '',
    loading: false,
  });

  const setErrorMessage = message => {
    setResponseData(prevResponseData => ({ ...prevResponseData, errorMessage: message }));
  };

  const handleSubmit = async (): Promise<void> => {
    let loadingToastTimer;
    try {
      if (showReactEditor) {
        await executeTypescriptCode(code);
      } else {
        loadingToastTimer = setTimeout(() => {
          toast.loading('Rendering... It can take a while');
        }, 5000);

        setResponseData({ imageUrl: '', errorMessage: '', loading: true });
        if (scene instanceof Error) {
          throw new Error(`${scene.name};\n${scene.message}`);
        }
        const request = {
          scene: scene,
          inputs: inputResolutionsToResolutions(inputResolutions),
          output: outputResolution,
        };
        const blob = await renderImage({ ...request });
        const imageObjectURL = URL.createObjectURL(blob);
        toast.dismiss();
        clearTimeout(loadingToastTimer);

        setResponseData({ imageUrl: imageObjectURL, errorMessage: '', loading: false });
      }
    } catch (error: any) {
      let errorDescription;
      if (error instanceof ApiError && !error.response) {
        errorDescription = 'Failed to connect to the server!';
      } else {
        errorDescription = error.message;
      }
      setErrorMessage(errorDescription);
      if (loadingToastTimer) {
        toast.dismiss();
        clearTimeout(loadingToastTimer);
      }
      toast.error(`${errorDescription}`);
    }
  };

  useEffect(() => {
    const ifReactMode = new URLSearchParams(window.location.search).get('mode') === 'react';
    setShowReactEditor(ifReactMode);
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
          <PlaygroundSettings
            onSubmit={handleSubmit}
            sceneValidity={!(scene instanceof Error) || showReactEditor}
            onInputResolutionChange={updateInputResolutions}
            onOutputResolutionChange={(resolution: Resolution) => {
              setOutputResolution(resolution);
            }}
            inputsSettings={inputResolutions}
            outputResolution={outputResolution}
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
