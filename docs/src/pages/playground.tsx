import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import { useState } from 'react';
import toast, { Toaster } from 'react-hot-toast';
import { renderImage } from '../api';
import PlaygroundCodeEditor from '../components/PlaygroundCodeEditor';
import PlaygroundPreview from '../components/PlaygroundPreview';
import PlaygroundRenderSettings from '../components/PlaygroundRenderSettings';
import { AVAILABLE_RESOLUTIONS, InputResolutions, Resolution } from './Resolution';
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

const INITIAL_INPUTS_RESOLUTIONS: InputResolutions = {
  input_1: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
  input_2: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
  input_3: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
  input_4: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
  input_5: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
  input_6: AVAILABLE_RESOLUTIONS.Resoultion1920x1080,
};

const INITIAL_SCENE_STRING = JSON.stringify(INITIAL_SCENE, null, 2);

function Homepage() {
  const [scene, setScene] = useState<object | Error>(INITIAL_SCENE);
  const [inputsResolutions, setInputsResolutions] = useState<InputResolutions>(
    INITIAL_INPUTS_RESOLUTIONS
  );
  function updateInputsResolutions(inputId: string, resolution: Resolution) {
    setInputsResolutions({
      ...inputsResolutions,
      [inputId]: { ...resolution },
    });
  }

  const [responseData, setResponseData] = useState({
    imageUrl: '',
    errorMessage: '',
  });

  const setErrorMessage = message => {
    setResponseData(prevResponseData => ({ ...prevResponseData, errorMessage: message }));
  };

  const handleSubmit = async (): Promise<void> => {
    try {
      if (scene instanceof Error) {
        throw new Error(`${scene.name};\n${scene.message}`);
      }
      const request = {
        scene: scene,
        inputs: inputsResolutions,
      };
      const blob = await renderImage({ ...request });
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
        <div className={styles.codeEditorBox}>
          <PlaygroundCodeEditor
            onChange={setScene}
            initialCodeEditorContent={INITIAL_SCENE_STRING}
          />
        </div>
      </div>
      <div className={styles.rightSide}>
        <div className={styles.preview}>
          <PlaygroundPreview {...responseData} />
        </div>
        <div className={styles.settingsBox}>
          <PlaygroundRenderSettings onSubmit={handleSubmit} onChange={updateInputsResolutions} />
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
