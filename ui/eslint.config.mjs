import js from '@eslint/js';
import typescriptEslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';

export default typescriptEslint.config(
  {
    name: 'browser-agent/base',
    files: ['**/*.{ts,tsx}'],
    ignores: [
      'node_modules/**',
      'dist/**',
      'build/**',
      'ui/dist/**',
      '**/*.config.mjs',
      'ui/eslint.config.mjs',
      'ui/vite.config.ts',
      'src/**/*.test.ts',
      'src/**/*.spec.ts',
      'src/test/setup.ts',
      'vite.config.ts',
      '**/vite.config.ts',
    ],
    extends: [js.configs.recommended],
    rules: {
      // BA-COPY-001: lint first-party UI strings for marketing filler.
      'no-warning-comments': 'off',
    },
  },
  {
    name: 'browser-agent/ignores',
    ignores: ['src/test/setup.ts', 'vite.config.ts', '**/vite.config.ts'],
  },
  {
    name: 'browser-agent/react-hooks',
    files: ['**/*.{ts,tsx}'],
    extends: [reactHooks.configs['recommended-latest']],
    rules: {
      'react-hooks/exhaustive-deps': 'error',
    },
  },
  {
    name: 'browser-agent/typescript',
    files: ['**/*.{ts,tsx}'],
    extends: [
      typescriptEslint.configs.recommendedTypeChecked,
      typescriptEslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/strict-boolean-expressions': 'error',
    },
  },
);
