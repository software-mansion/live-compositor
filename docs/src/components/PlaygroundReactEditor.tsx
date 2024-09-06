import Editor, { Monaco } from '@monaco-editor/react';
import React, { useCallback, useLayoutEffect, useRef } from 'react';
import { useColorMode } from '@docusaurus/theme-common';
import { tsCompilerOptions } from '../monacoEditorConfig';
import { usePluginData } from '@docusaurus/useGlobalData';

function useEvent<TFunction extends (...params: any[]) => any>(handler: TFunction) {
  const handlerRef = useRef(handler);

  // In a real implementation, this would run before layout effects
  useLayoutEffect(() => {
    handlerRef.current = handler;
  });

  return useCallback((...args: Parameters<TFunction>) => {
    // In a real implementation, this would throw if called during render
    const fn = handlerRef.current;
    return fn(...args);
  }, []) as TFunction;
}

async function handleEditorWillMount(monaco: Monaco, pathsToTypeFiles: string[]) {
  const tsDefaults = monaco?.languages.typescript.typescriptDefaults;

  for (const pathToTypeFile of pathsToTypeFiles) {
    await fetch(pathToTypeFile)
      .then(response => response.text())
      .then(fileContent => {
        tsDefaults.addExtraLib(fileContent, pathToTypeFile.replace('/playground-types/', ''));
      });
  }

  tsDefaults.setCompilerOptions({
    ...tsCompilerOptions(),
  });
}

type Props = {
  code: string;
  onCodeChange: (value: string) => unknown;
};

export default function PlaygroundReactEditor(props: Props) {
  const { code, onCodeChange } = props;
  const { colorMode } = useColorMode();
  const pluginData = usePluginData('copy-type-files-plugin');
  const pathsToTypeFiles = pluginData['pathsToTypeFiles'];

  const handleChange = useEvent((value: string | undefined) => {
    onCodeChange(value ?? '');
  });

  return (
    <Editor
      height="100%"
      width="100%"
      className="monacoEditor"
      defaultLanguage="typescript"
      value={code}
      onChange={handleChange}
      beforeMount={monaco => handleEditorWillMount(monaco, pathsToTypeFiles)}
      defaultPath="file:///main.tsx"
      theme={colorMode === 'dark' ? 'vs-dark' : 'light'}
    />
  );
}
