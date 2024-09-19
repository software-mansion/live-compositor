import { Choice, selectPrompt, textPrompt } from './utils/prompts';
import path from 'path';
import { PackageManager } from './utils/packageManager';

export type ProjectOptions = {
  projectName: string;
  directory: string;
  packageManager: PackageManager;
  runtime: BrowserOptions | NodeOptions;
};

type BrowserOptions = {
  type: 'browser';
  embeddedWasm: boolean;
  templateName: 'vite' | 'next';
};

type NodeOptions = {
  type: 'node';
  templateName: string;
};

type Runtime = 'node' | 'browser';

const packageManagers: Choice<PackageManager>[] = [
  { value: 'npm', title: 'npm' },
  { value: 'yarn', title: 'yarn' },
  { value: 'pnpm', title: 'pnpm' },
];

export async function resolveOptions(): Promise<ProjectOptions> {
  const projectName = await textPrompt('Project name: ', 'live-compositor-app');
  // TODO: replace
  // const runtime = await selectPrompt('Select environment:', [
  //   { value: 'node', title: 'Node.js' },
  //   { value: 'browser', title: 'Browser' },
  // ] as const);
  const runtime: Runtime = 'node' as any;

  const packageManager = await selectPrompt('Select package manager: ', packageManagers);

  let runtimeOptions: ProjectOptions['runtime'];
  if (runtime === 'browser') {
    runtimeOptions = await resolveBrowserOptions();
  } else if (runtime === 'node') {
    runtimeOptions = await resolveNodeOptions();
  } else {
    throw new Error('Unknown runtime');
  }

  return {
    runtime: runtimeOptions,
    packageManager,
    projectName,
    directory: path.join(process.cwd(), projectName),
  };
}

export async function resolveBrowserOptions(): Promise<BrowserOptions> {
  const usageType = await selectPrompt('Where do you want to run the LiveCompositor server?', [
    { value: 'external', title: 'Run as an external instance and communicate over the network.' },
    { value: 'wasm', title: 'Embed LiveCompositor in the browser and render using WebGL.' },
  ]);
  const templateName = await selectPrompt('Select project template:', [
    { value: 'vite', title: 'Vite + React' },
    { value: 'next', title: 'Next.js' },
  ] as const);

  return {
    type: 'browser',
    embeddedWasm: usageType === 'wasm',
    templateName,
  };
}

export async function resolveNodeOptions(): Promise<NodeOptions> {
  const templateName = await selectPrompt('Select project template: ', [
    { title: 'minimal', value: 'node-minimal' },
    { title: 'Express.js + Redux', value: 'node-express-redux' },
  ] as const);
  return {
    type: 'node',
    templateName,
  };
}
