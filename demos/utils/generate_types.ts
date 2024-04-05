import * as fs from 'fs';
import * as path from 'path';
import { compileFromFile } from 'json-schema-to-typescript'

async function generateTypes() {
    const schemaPath = path.resolve(__dirname, '../../schemas/request.schema.json');
    const tsOutputPath = path.resolve(__dirname, '../types/types.ts');

    const registerTs = await compileFromFile(schemaPath);
    fs.writeFileSync(tsOutputPath, registerTs);
}

generateTypes();
