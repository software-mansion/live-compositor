import { defaultJsonExample } from '@site/src/scene/defaultJsonExample';
import { videoCallExample } from '@site/src/scene/videoCallExample';
import { removeGreenScreenExample } from '@site/src/scene/removeGreenScreenExample';
import { Api } from 'live-compositor';

export default function PlaygroundSettingsExamples({
  populateEditorWithExample,
  closeModal,
}: {
  populateEditorWithExample: (content: object | Error) => void;
  closeModal: () => void;
}) {
  return (
    <div className="flex flex-nowrap flex-col p-[2px_4px_2px_4px] divide-y divide-x-0 divide-solid divide-[var(--ifm-color-emphasis-300)]">
      <ExampleInfo
        exampleName="Default example"
        description="Showcase of various components and layout mechanism combined with custom shaders."
        jsonExample={defaultJsonExample}
        populateEditorWithExample={populateEditorWithExample}
        closeModal={closeModal}
      />
      <ExampleInfo
        exampleName="Video call"
        description="Grid layout commonly used to display participants of a video call. One tile has a border rendered with a shader. One tile renders an image instead of a stream."
        jsonExample={videoCallExample}
        populateEditorWithExample={populateEditorWithExample}
        closeModal={closeModal}
      />
      <ExampleInfo
        exampleName="Greenscreen"
        description="Image of a man (with shader replacing greenscreen) with live-compositor logo in the top-left corner."
        jsonExample={removeGreenScreenExample}
        populateEditorWithExample={populateEditorWithExample}
        closeModal={closeModal}
      />
    </div>
  );
}

interface ExampleInfoProps {
  exampleName: string;
  description: string;
  jsonExample: () => Api.Component;
  populateEditorWithExample: (content: object | Error) => void;
  closeModal: () => void;
}

function ExampleInfo({
  exampleName,
  description,
  jsonExample,
  populateEditorWithExample,
  closeModal,
}: ExampleInfoProps) {
  return (
    <div>
      <div className=" flex">
        <div className="flex-[0_0_20%] flex font-semibold text-center py-3 min-w-5 justify-center items-center">
          {exampleName}
        </div>
        <div className="flex-[0_0_70%] flex text-left py-3 px-5 items-center">{description}</div>
        <div className="flex-[0_0_10%] flex py-3 justify-center items-center">
          <button
            className="rounded-md px-2 font-bold bg-[var(--docsearch-primary-color)] text-lg text-[var(--ifm-color-emphasis-0)] border-solid border border-[var(--ifm-color-primary)]"
            onClick={() => {
              populateEditorWithExample(jsonExample());
              closeModal();
            }}>
            Run
          </button>
        </div>
      </div>
    </div>
  );
}
