import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

// eslint-disable-next-line @typescript-eslint/no-var-requires
export const languages = ExecutionEnvironment.canUseDOM ? require('monaco-editor').languages : null;
