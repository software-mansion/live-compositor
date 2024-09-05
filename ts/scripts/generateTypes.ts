import fs from 'fs';
import * as path from 'path';
import { compileFromFile } from 'json-schema-to-typescript';

async function generateTypes() {
  const schemaPath = path.resolve(__dirname, '../../schemas/api_types.schema.json');
  const tsOutputPath = path.resolve(__dirname, '../live-compositor/src/api.generated.ts');

  const typesTs = await compileFromFile(schemaPath, {
    additionalProperties: false,
  });
  fs.writeFileSync(tsOutputPath, typesTs);
}

generateTypes();
