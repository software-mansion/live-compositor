import { example01 } from '@site/src/scene/jsonExample01';
import { example02 } from '@site/src/scene/jsonExample02';
import { example03 } from '@site/src/scene/jsonExample03';

export default function PlaygroundSettingsExamples({
  setExample,
  closeModal,
}: {
  setExample: (content: object | Error) => void;
  closeModal: () => void;
}) {
  return (
    <div className="flex flex-nowrap flex-col p-[4px_8px_4px_8px]">
      <ExampleInfo
        example_name="Example 01"
        description="Video call scene with one main window (with shader removing greenscreen) and side bar with the rest participants' windows."
        setExample={setExample}
        closeModal={closeModal}
      />
      <ExampleInfo
        example_name="Example 02"
        description="Scene with windows in grid layout on white background. Done using tiles component."
        setExample={setExample}
        closeModal={closeModal}
      />
      <ExampleInfo
        example_name="Example 03"
        description="Image of the man (with shader removing greenscreen) with live-compositor logo in the top-left corner."
        setExample={setExample}
        closeModal={closeModal}
      />
    </div>
  );
}

interface ExampleInfoProps {
  example_name: string;
  description: string;
  setExample: (content: object | Error) => void;
  closeModal: () => void;
}

function ExampleInfo({ example_name, description, setExample, closeModal }: ExampleInfoProps) {
  return (
    <div className="flex m-4 border-b-solid border-t-solid">
      <div className="flex-[0_0_20%] text-center font-bold text-xl">{example_name}</div>
      <div className="flex-[0_0_2%]" />
      <div className="flex-[0_0_60%]">{description}</div>
      <div className="flex-[0_0_3%]" />
      <button
        className="flex-[0_0_15%] rounded-xl font-bold bg-[var(--docsearch-primary-color)] text-lg h-16 text-[var(--ifm-color-emphasis-0)] border-solid border border-[var(--ifm-color-primary)]"
        onClick={() => {
          if (example_name === 'Example 01') {
            setExample(example01());
          } else if (example_name === 'Example 02') {
            setExample(example02());
          } else {
            setExample(example03());
          }
          closeModal();
        }}>
        Use example
      </button>
    </div>
  );
}
