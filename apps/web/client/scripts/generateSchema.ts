import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { z } from 'zod';
import { repomixConfigFileSchema } from '../../../../legacy/repomix-ts/src/config/configSchema.js';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../../..');
const legacyPackageDir = path.join(repoRoot, 'legacy/repomix-ts');

const getPackageVersion = async (): Promise<string> => {
  const packageJsonPath = path.join(legacyPackageDir, 'package.json');
  const packageJsonContent = await fs.readFile(packageJsonPath, 'utf-8');
  const packageJson = JSON.parse(packageJsonContent);
  return packageJson.version;
};

const generateSchema = async () => {
  const version = await getPackageVersion();
  const versionParts = version.split('.');
  const majorMinorVersion = `${versionParts[0]}.${versionParts[1]}.${versionParts[2]}`;

  // Use Zod v4's built-in JSON Schema generation
  const jsonSchema = z.toJSONSchema(repomixConfigFileSchema, {
    target: 'draft-7',
  });

  const schemaWithMeta = {
    $schema: 'http://json-schema.org/draft-07/schema#',
    ...jsonSchema,
    title: 'Repomix Configuration',
    description: 'Schema for repomix.config.json configuration file',
  };

  const baseOutputDir = path.join(repoRoot, 'apps/web/client/src/public/schemas');
  await fs.mkdir(baseOutputDir, { recursive: true });

  const versionedOutputDir = path.join(baseOutputDir, majorMinorVersion);
  await fs.mkdir(versionedOutputDir, { recursive: true });

  const versionedOutputPath = path.join(versionedOutputDir, 'schema.json');
  await fs.writeFile(versionedOutputPath, JSON.stringify(schemaWithMeta, null, 2), 'utf-8');

  const latestOutputDir = path.join(baseOutputDir, 'latest');
  await fs.mkdir(latestOutputDir, { recursive: true });
  const latestOutputPath = path.join(latestOutputDir, 'schema.json');
  await fs.writeFile(latestOutputPath, JSON.stringify(schemaWithMeta, null, 2), 'utf-8');

  console.log(`Schema generated at ${versionedOutputPath}`);
  console.log(`Schema also generated at ${latestOutputPath}`);
};

generateSchema().catch(console.error);
