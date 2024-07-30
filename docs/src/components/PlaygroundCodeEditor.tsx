import styles from './PlaygroundCodeEditor.module.css';
import { ChangeEvent, useState } from 'react';

interface PlaygroundCodeEditorProps {
  onChange: (content: object | Error) => void;
  initialCodeEditorContent: string;
}

function PlaygroundCodeEditor({ onChange, initialCodeEditorContent }: PlaygroundCodeEditorProps) {
  const [content, setContent] = useState<string>(initialCodeEditorContent);

  const handleChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    const codeContent = event.target.value;
    setContent(codeContent);
    try {
      const scene = JSON.parse(codeContent);
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
      value={content}
      onChange={handleChange}
    />
  );
}
export default PlaygroundCodeEditor;
