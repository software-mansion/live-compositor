import React, { useEffect, useRef } from 'react';
import JSONEditor, { Ajv } from 'jsoneditor';
import 'jsoneditor/dist/jsoneditor.css';
import './jsoneditor-dark.css';

const JSON_SCHEMA_FOR_DEV = {
  $schema: 'http://json-schema.org/draft-07/schema#',
  title: 'User',
  type: 'object',
  properties: {
    name: {
      type: 'string',
      minLength: 1,
    },
    age: {
      type: 'number',
      minimum: 0,
      format: 'float',
    },
    email: {
      type: 'string',
    },
    address: {
      type: 'object',
      properties: {
        street: {
          type: 'string',
        },
        city: {
          type: 'string',
        },
        state: {
          type: 'string',
        },
        zip: {
          type: 'string',
          pattern: '^[0-9]{5}(?:-[0-9]{4})?$',
        },
      },
      required: ['street', 'city', 'state', 'zip'],
    },
  },
  required: ['name', 'age', 'email'],
};

interface PlaygroundCodeEditorProps {
  onChange: (content: object | Error) => void;
  initialCodeEditorContent: object;
}

function PlaygroundCodeEditor({ onChange, initialCodeEditorContent }: PlaygroundCodeEditorProps) {
  const editorContainer = useRef<HTMLDivElement | null>(null);
  const jsonEditor = useRef<JSONEditor | null>(null);

  const ajv = ajvInitialization();

  useEffect(() => {
    if (editorContainer.current) {
      jsonEditor.current = new JSONEditor(editorContainer.current, {
        mode: 'code',
        enableSort: false,
        enableTransform: false,
        statusBar: false,
        mainMenuBar: false,
        ajv,
        onChange: () => {
          if (jsonEditor.current) {
            try {
              const jsonContent = jsonEditor.current.get();
              onChange(jsonContent);
            } catch (error) {
              onChange(error);
            }
          }
        },
      });

      jsonEditor.current.setSchema(JSON_SCHEMA_FOR_DEV);

      try {
        jsonEditor.current.set(initialCodeEditorContent);
      } catch (error) {
        onChange(error);
      }
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

function ajvInitialization() {
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
