import React, { useState, useEffect, useCallback } from 'react';
import 'jsoneditor/dist/jsoneditor.css';
import './jsoneditor-dark.css';
import componentTypesJsonSchema from '../../component_types.schema.json';
import { JSONEditor } from '../playgroundCodeEditorUtils';

interface PlaygroundCodeEditorProps {
  onChange: (content: object | Error) => void;
  initialCodeEditorContent: object;
}

function ajvInitialization() {
  const ajv = JSONEditor.Ajv({
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

function PlaygroundCodeEditor({ onChange, initialCodeEditorContent }: PlaygroundCodeEditorProps) {
  const [jsonEditor, setJsonEditor] = useState<typeof JSONEditor | null>(null);

  const editorContainer = useCallback(node => {
    if (node === null) {
      return;
    }
    const ajv = ajvInitialization();
    const validate = ajv.compile(componentTypesJsonSchema);

    const editor = new JSONEditor(node, {
      mode: 'code',
      enableSort: false,
      enableTransform: false,
      statusBar: false,
      mainMenuBar: false,
      ajv,
      onChange: () => {
        try {
          const jsonContent = editor.get();
          onChange(jsonContent);
          if (!validate(jsonContent)) {
            throw new Error('Invalid JSON!');
          }
        } catch (error) {
          onChange(error);
        }
      },
      onBlur: () => {
        try {
          editor.format();
        } catch (error) {
          onChange(error);
        }
      },
    });

    editor.setSchema(componentTypesJsonSchema);
    editor.set(initialCodeEditorContent);

    setJsonEditor(editor);
  }, []);

  useEffect(() => {
    return () => {
      if (jsonEditor) {
        jsonEditor.destroy();
      }
    };
  }, [jsonEditor]);

  return <div ref={editorContainer} style={{ height: '100%' }} />;
}

export default PlaygroundCodeEditor;
