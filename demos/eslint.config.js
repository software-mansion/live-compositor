import globals from 'globals';

import eslintRecommended from '@eslint/js';
import eslintConfigPrettier from 'eslint-config-prettier';

import pluginImport from 'eslint-plugin-import';
import pluginPrettierRecommended from 'eslint-plugin-prettier/recommended';
import { plugin as tsEslintPlugin } from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
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
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    settings: {
      'import/parsers': {
        '@typescript-eslint/parser': ['.ts', '.tsx'],
      },
      'import/resolver': {
        typescript: {
          alwaysTryTypes: true,
          project: 'tsconfig.json',
        },
      },
    },
    rules: {
      'prettier/prettier': ['error'],
      'import/no-unresolved': 'error',
      '@typescript-eslint/no-explicit-any': [0, {}],
      '@typescript-eslint/no-floating-promises': ['error'],
      'no-constant-condition': [0],
      'no-unused-vars': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          caughtErrors: 'all',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
          vars: 'local',
        },
      ],
      '@typescript-eslint/consistent-type-imports': [
        'error',
        {
          prefer: 'type-imports',
        },
      ],
    },
  },
  {
    files: ['**/*.{ts,tsx}'],
    plugins: {
      'react-hooks': reactHooks,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
    },
  },
  {
    ignores: ['eslint.config.js', '**/*.d.ts'],
  },
];
