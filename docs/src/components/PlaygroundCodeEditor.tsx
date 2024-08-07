/* eslint @typescript-eslint/no-var-requires: "off" */

import React, { useEffect, useRef } from 'react';
import 'jsoneditor/dist/jsoneditor.css';
import './jsoneditor-dark.css';
import component_types_json_schema from '../../../schemas/component_types.schema.json';

function ajvInitialization(Ajv) {
  const ajv = Ajv({
    allErrors: true,
    verbose: true,
    schemaId: 'auto',
    $data: true,
  });

  ajv.addFormat('float', '^-?d+(.d+)?([eE][+-]?d+)?$');
  ajv.addFormat('double', '^-?d+(.d+)?([eE][+-]?d+)?$');
  ajv.addFormat('int32', '^-?d+$');
  ajv.addFormat('uint32', '^d+$');
  ajv.addFormat('uint', '^d+$');

  return ajv;
}

interface PlaygroundCodeEditorProps {
  onChange: (content: object | Error) => void;
  initialCodeEditorContent: object;
}

function PlaygroundCodeEditor({ onChange, initialCodeEditorContent }: PlaygroundCodeEditorProps) {
  const editorContainer = useRef<HTMLDivElement | null>(null);
  const jsonEditor = useRef(null);

  useEffect(() => {
    const JSONEditor = require('jsoneditor');
    const Ajv = require('jsoneditor').Ajv;
    if (editorContainer.current) {
      const ajv = ajvInitialization(Ajv);
      const validate = ajv.compile(component_types_json_schema);
      jsonEditor.current = new JSONEditor(editorContainer.current, {
        mode: 'code',
        enableSort: false,
        enableTransform: false,
        statusBar: false,
        mainMenuBar: false,
        ajv,
        onChange: () => {
          try {
            const jsonContent = jsonEditor.current.get();
            onChange(jsonContent);
            if (!validate(jsonContent)) throw new Error('Invalid JSON!');
          } catch (error) {
            onChange(error);
          }
        },
      });

      jsonEditor.current.setSchema(component_types_json_schema);
      jsonEditor.current.set(initialCodeEditorContent);
    }

    return () => {
      if (jsonEditor.current) {
        jsonEditor.current.destroy();
      }
    };
  }, []);
  return <div ref={editorContainer} style={{ height: '100%' }} />;
}
export default PlaygroundCodeEditor;
