import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../../..');

const generateSchema = async () => {
  const baseOutputDir = path.join(repoRoot, 'apps/web/client/src/public/schemas');
  const latestOutputPath = path.join(baseOutputDir, 'latest', 'schema.json');
  const latestSchemaContent = await fs.readFile(latestOutputPath, 'utf-8');
  JSON.parse(latestSchemaContent);

  console.log(`Verified checked-in schema at ${latestOutputPath}`);
  console.log('Legacy TypeScript schema generator has been removed; keep schemas in sync manually.');
};

generateSchema().catch(console.error);
