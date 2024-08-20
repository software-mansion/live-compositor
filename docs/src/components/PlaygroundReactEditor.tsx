import Editor, { Monaco } from '@monaco-editor/react';
import React, { useCallback, useLayoutEffect, useRef } from 'react';
import { useColorMode } from '@docusaurus/theme-common';
import { tsCompilerOptions } from '../monacoEditorConfig';
import BrowserOnly from '@docusaurus/BrowserOnly';

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

function handleEditorWillMount(monaco: Monaco) {
  const tsDefaults = monaco?.languages.typescript.typescriptDefaults;

  fetch('https://unpkg.com/@types/react/index.d.ts')
    .then(response => response.text())
    .then(reactTypes => {
      tsDefaults.addExtraLib(reactTypes, 'react/index.d.ts');
    });

  fetch('https://unpkg.com/@types/react/jsx-runtime.d.ts')
    .then(response => response.text())
    .then(reactTypes => {
      tsDefaults.addExtraLib(reactTypes, 'react/jsx-runtime.d.ts');
    });

  fetch('https://unpkg.com/@types/react-dom/index.d.ts')
    .then(response => response.text())
    .then(reactDOMTypes => {
      tsDefaults.addExtraLib(reactDOMTypes, 'react-dom/index.d.ts');
    });

  tsDefaults.setCompilerOptions({
    ...tsCompilerOptions,
  });
}

type Props = {
  code: string;
  onCodeChange: (value: string) => unknown;
};

export default function PlaygroundReactEditor(props: Props) {
  const { code, onCodeChange } = props;
  const { colorMode, setColorMode } = useColorMode();

  const handleChange = useEvent((value: string | undefined) => {
    onCodeChange(value ?? '');
  });

  return (
    <BrowserOnly>
      {() => (
        <div style={{ height: '100%', border: 'solid', borderColor: 'grey' }}>
          <Editor
            height="93%"
            defaultLanguage="typescript"
            value={code}
            onChange={handleChange}
            beforeMount={handleEditorWillMount}
            defaultPath="file:///main.tsx"
            theme={colorMode === 'dark' ? 'vs-dark' : 'light'}
          />
        </div>
      )}
    </BrowserOnly>
  );
}
