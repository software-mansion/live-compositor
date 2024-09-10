import fs from 'fs-extra';
import path from 'path';

const TEMPLATES_ROOT = path.join(__dirname, '../templates');

export async function applyTemplate(
  destination: string,
  templateName: string,
  projectName: string
): Promise<void> {
  const templatePath = path.join(TEMPLATES_ROOT, templateName);
  await fs.copy(templatePath, destination);

  await fs.remove(path.join(destination, 'node_modules'));
  await fs.remove(path.join(destination, 'dist'));

  const packageJsonPath = path.join(destination, 'package.json');
  const packageJson = JSON.parse(await fs.readFile(packageJsonPath, 'utf8'));
  delete packageJson?.scripts?.['start'];
  delete packageJson['private'];
  packageJson.name = projectName;
  await fs.writeFile(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n', 'utf8');
}
