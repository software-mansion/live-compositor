import { languages } from './monacoEditor';

export function tsCompilerOptions() {
  return {
    target: languages.typescript.ScriptTarget.ESNext,
    allowNonTsExtensions: true,
    strict: true,
    esModuleInterop: true,
    moduleResolution: languages.typescript.ModuleResolutionKind.NodeJs,
    jsx: languages.typescript.JsxEmit.React,
    skipLibCheck: true,
    exactOptionalPropertyTypes: true,
    baseUrl: '.',
    lib: ['dom', 'es2021'],
    allowJs: true,
    isolatedModules: true,
    noEmit: true,
  };
}
