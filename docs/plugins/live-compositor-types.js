import fs from 'fs';
import path from 'path';

export default function copyTypeFilesPlugin() {
  const srcDirLiveCompositor = './node_modules/live-compositor/dist';
  const destDirLiveCompositor = './static/playground-types/live-compositor';

  const srcDirReact = './node_modules/@types/react';
  const destDirReact = './static/playground-types/react';

  const dirPlaygroundTypes = './static/playground-types';

  if (!fs.existsSync(dirPlaygroundTypes)) {
    fs.mkdirSync(dirPlaygroundTypes);
  }
  if (!fs.existsSync(destDirLiveCompositor)) {
    fs.mkdirSync(destDirLiveCompositor);
  }
  if (!fs.existsSync(destDirReact)) {
    fs.mkdirSync(destDirReact);
  }

  fs.cpSync(srcDirLiveCompositor, destDirLiveCompositor, { recursive: true });
  fs.cpSync(path.join(srcDirReact, 'index.d.ts'), path.join(destDirReact, 'index.d.ts'));

  const pathsToTypeFiles = findDTSFiles('./static/playground-types');

  return {
    name: 'copy-type-files-plugin',
    loadContent() {
      return pathsToTypeFiles;
    },
    contentLoaded({ content, actions }) {
      const { setGlobalData } = actions;
      setGlobalData({ pathsToTypeFiles: content });
    },
  };
}

function findDTSFiles(dir) {
  let pathsToTypeFiles = [];
  const files = fs.readdirSync(dir);

  for (const file of files) {
    const filePath = path.join(dir, file);
    const fileStat = fs.statSync(filePath);

    if (fileStat.isDirectory()) {
      pathsToTypeFiles = pathsToTypeFiles.concat(findDTSFiles(filePath));
    } else if (fileStat.isFile() && file.endsWith('.d.ts')) {
      pathsToTypeFiles.push(filePath.replace('static', ''));
    }
  }
  return pathsToTypeFiles;
}

module.exports = copyTypeFilesPlugin;
