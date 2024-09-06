import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const JSONEditor = ExecutionEnvironment.canUseDOM ? require('jsoneditor') : null;

export default JSONEditor;
