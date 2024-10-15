// {
//   "env": {
//     "node": true
//   },
//   "extends": [
//     "eslint:recommended",
//     "plugin:@typescript-eslint/eslint-recommended",
//     "plugin:@typescript-eslint/recommended",
//     "plugin:import/recommended",
//     "plugin:import/typescript",
//     "prettier"
//   ],
//   "plugins": [
//     "prettier"
//   ],
//   "parser": "@typescript-eslint/parser",
//   "parserOptions": {
//     "project": [
//       "tsconfig.json"
//     ]
//   },
//   "rules": {
//     "prettier/prettier": ["error"],
//     "@typescript-eslint/no-explicit-any": [0, {}],
//     "@typescript-eslint/no-floating-promises": "error",
//     "no-constant-condition": [0]
//   }
// }
import eslintRecommended from '@eslint/js';
import eslintConfigPrettier from 'eslint-config-prettier';

import pluginImport from 'eslint-plugin-import';
import pluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import { plugin as tsEslintPlugin } from 'typescript-eslint';

import tsParser from '@typescript-eslint/parser';

export default [
  eslintRecommended.configs.recommended,
  pluginImport.flatConfigs.recommended,
  pluginPrettierRecommended,
  eslintConfigPrettier,
  {
    files: ['**/*.{js,jsx,ts,tsx}'],
    ignores: ['.prettierrc.js'],
    plugins: {
      '@typescript-eslint': tsEslintPlugin,
    },
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: 'tsconfig.json',
      },
    },
    settings: {
      'import/parsers': {
        '@typescript-eslint/parser': ['.ts', '.tsx'],
      },
      'import/resolver': {
        typescript: {
          alwaysTryTypes: true,
          project: '**/tsconfig.json',
        },
      },
    },
    rules: {
      'prettier/prettier': ['error'],
      '@typescript-eslint/no-explicit-any': [0, {}],
      '@typescript-eslint/no-floating-promises': 'error',
      'no-constant-condition': [0],
    },
  },
];
