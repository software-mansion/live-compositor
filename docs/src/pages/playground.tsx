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
import { example01 } from '@site/src/scene/jsonExample01';
import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

const STORED_CODE_EDITOR_CONTENT = ExecutionEnvironment.canUseDOM
  ? sessionStorage.getItem('playgroundCodeEditorContent')
  : null;

const INITIAL_SCENE =
  STORED_CODE_EDITOR_CONTENT !== null ? JSON.parse(STORED_CODE_EDITOR_CONTENT) : example01();

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
  const [example, setExample] = useState<object>(INITIAL_SCENE);
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
    setResponseData(prevResponseData => ({
      ...prevResponseData,
      errorMessage: message,
      loading: false,
    }));
  };

  const handleSubmit = async (): Promise<void> => {
    let loadingToastTimer;
    let loadingToast;
    try {
      if (showReactEditor) {
        await executeTypescriptCode(code);
      } else {
        loadingToastTimer = setTimeout(() => {
          loadingToast = toast.loading('Rendering... It can take a while');
        }, 3000);

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
        if (loadingToast) toast.dismiss(loadingToast);
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

  function updateExample(content: object) {
    setScene(content);
    setExample(content);
  }

  useEffect(() => {
    handleSubmit();
  }, [example]);

  return (
    <div className="flex flex-row flex-wrap p-8 lg:h-[calc(100vh-110px)]">
      <div className="flex-1 m-2 border-2 border-gray-400 border-solid rounded-md max-h-full min-w-[300px] min-h-[500px]">
        {showReactEditor ? (
          <PlaygroundReactEditor code={code} onCodeChange={setCode} />
        ) : (
          <PlaygroundCodeEditor onChange={setScene} codeExample={example} />
        )}
      </div>
      <div className="flex flex-col flex-1 max-h-full">
        <div className="flex flex-1 m-2 justify-center border-2 border-gray-400 border-solid rounded-md min-w-[300px] min-h-[120px]">
          <PlaygroundPreview {...responseData} />
        </div>
        <div className="flex flex-1 m=2 min-w-[300px] min-h-[300px]">
          <PlaygroundSettings
            onSubmit={handleSubmit}
            isLoading={responseData.loading}
            sceneValidity={!(scene instanceof Error) || showReactEditor}
            onInputResolutionChange={updateInputResolutions}
            onOutputResolutionChange={(resolution: Resolution) => {
              setOutputResolution(resolution);
            }}
            inputsSettings={inputResolutions}
            outputResolution={outputResolution}
            setExample={updateExample}
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
