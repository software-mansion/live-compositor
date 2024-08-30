import Editor, { Monaco } from '@monaco-editor/react';
import React, { useCallback, useLayoutEffect, useRef } from 'react';
import { useColorMode } from '@docusaurus/theme-common';
import { tsCompilerOptions } from '../monacoEditorConfig';

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

async function handleEditorWillMount(monaco: Monaco) {
  const tsDefaults = monaco?.languages.typescript.typescriptDefaults;

  await fetch('https://unpkg.com/@types/react/index.d.ts')
    .then(response => response.text())
    .then(reactTypes => {
      tsDefaults.addExtraLib(reactTypes, 'react/index.d.ts');
    });

  await fetch('https://unpkg.com/@types/react/jsx-runtime.d.ts')
    .then(response => response.text())
    .then(reactTypes => {
      tsDefaults.addExtraLib(reactTypes, 'react/jsx-runtime.d.ts');
    });

  await fetch('https://unpkg.com/live-compositor@0.1.0-rc.1/src/index.ts')
    .then(response => response.text())
    .then(reactDOMTypes => {
      tsDefaults.addExtraLib(reactDOMTypes, 'live-compositor/index.d.ts');
    });

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
      beforeMount={handleEditorWillMount}
      defaultPath="file:///main.tsx"
      theme={colorMode === 'dark' ? 'vs-dark' : 'light'}
    />
  );
}
