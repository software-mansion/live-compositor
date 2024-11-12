import { spawn } from './spawn';

export type PackageManager = 'npm' | 'yarn' | 'pnpm';

export async function runPackageManagerInstall(pm: PackageManager, cwd?: string): Promise<void> {
  const args: string[] = [];
  if (['pnpm', 'npm'].includes(pm)) {
    args.push('install');
  }
  await spawn(pm, args, {
    cwd: cwd ?? process.cwd(),
  });
}
