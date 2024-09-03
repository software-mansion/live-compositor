import fs from 'fs';
import * as path from 'path';
import { compileFromFile } from 'json-schema-to-typescript';

async function generateTypes() {
  const schemaPath = path.resolve(__dirname, '../../schemas/api_types.schema.json');
  const outputPaths = [
    path.resolve(__dirname, '../live-compositor/src/api.generated.ts'),
    path.resolve(__dirname, '../@live-compositor/browser-render/src/api.generated.ts')
  ];

  const typesTs = await compileFromFile(schemaPath, {
    additionalProperties: false,
  });

  for (const path of outputPaths) {
    fs.writeFileSync(path, typesTs);
  }
}

generateTypes();
