import type { ProjectOptions } from './options';
import { ensureProjectDir } from './utils/workingdir';
import { runPackageManagerInstall } from './utils/packageManager';
import { applyTemplate } from './template';

export async function createNodeProject(options: ProjectOptions) {
  await ensureProjectDir(options.directory);
  await applyTemplate(options.directory, options.runtime.templateName, options.projectName);
  await runPackageManagerInstall(options.packageManager, options.directory);
}
