import styles from '../pages/playground.module.css';
import { ChangeEvent, useState } from 'react';

interface PlaygroundCodeEditorProps {
  onChange: (content: object | Error) => void;
  initialCodeEditorContent: string;
}

function PlaygroundCodeEditor({
  onChange,
  initialCodeEditorContent,
}: PlaygroundCodeEditorProps): JSX.Element {
  const [currCodeEditorContent, setCurrCodeEditorContent] =
    useState<string>(initialCodeEditorContent);

  const handleChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    const currCode = event.target.value;
    setCurrCodeEditorContent(currCode);
    try {
      const scene = JSON.parse(currCode);
      onChange(scene);
    } catch (error) {
      onChange(error);
    }
  };

  return (
    <textarea
      className={styles.codeEditor}
      name="inputArea"
      placeholder="Enter your code to try it out"
      value={currCodeEditorContent}
      onChange={handleChange}
    />
  );
}
export default PlaygroundCodeEditor;
