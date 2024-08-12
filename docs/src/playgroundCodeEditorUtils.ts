import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';
import type JSONEditorType from 'jsoneditor';

/* eslint-disable-next-line @typescript-eslint/no-var-requires */
export const JSONEditor = (
  ExecutionEnvironment.canUseDOM ? require('jsoneditor') : null
) as JSONEditorType;

// module.exports = {
//   JSONEditor: (ExecutionEnvironment.canUseDOM ? require('jsoneditor') : null) as JSONEditorType,
// };
