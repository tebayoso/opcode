import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";
import globals from "globals";

export default tseslint.config(
  // Global ignores
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "*.config.js",
      "*.config.ts",
      "vite.config.ts",
    ],
  },

  // Base JavaScript rules
  js.configs.recommended,

  // TypeScript strict rules
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,

  // React configuration
  {
    files: ["**/*.{ts,tsx}"],
    plugins: {
      react,
      "react-hooks": reactHooks,
      "jsx-a11y": jsxA11y,
    },
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.es2021,
      },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        ecmaFeatures: {
          jsx: true,
        },
      },
    },
    settings: {
      react: {
        version: "detect",
      },
    },
    rules: {
      // React rules
      ...react.configs.recommended.rules,
      ...react.configs["jsx-runtime"].rules,
      ...reactHooks.configs.recommended.rules,

      // JSX Accessibility rules
      ...jsxA11y.configs.strict.rules,

      // React specific overrides
      "react/prop-types": "off", // TypeScript handles this
      "react/react-in-jsx-scope": "off", // Not needed with new JSX transform
      "react/jsx-no-target-blank": "error",
      "react/jsx-curly-brace-presence": ["error", { props: "never", children: "never" }],
      "react/self-closing-comp": "error",
      "react/jsx-boolean-value": ["error", "never"],
      "react/jsx-no-useless-fragment": "error",
      "react/hook-use-state": "off", // Allow non-destructured useState for refs
      "react/jsx-pascal-case": "error",
      "react/no-array-index-key": "warn",
      "react/jsx-handler-names": "off", // Too restrictive for existing codebase
      "react/display-name": "off", // Anonymous functions are acceptable
      "react/no-unescaped-entities": "off", // Allow apostrophes and quotes in JSX

      // React Hooks - keep only basic rules, disable all strict/compiler rules
      "react-hooks/rules-of-hooks": "error", // Keep rules of hooks as error
      "react-hooks/exhaustive-deps": "warn", // Warn for exhaustive deps
      "react-hooks/static-components": "off",
      "react-hooks/set-state-in-effect": "off",
      "react-hooks/ref-access": "off",
      "react-hooks/scope": "off",
      "react-hooks/purity": "off",
      "react-hooks/var-order": "off",
      "react-hooks/refs": "off",
      "react-hooks/use-before-define": "off",
      "react-hooks/use-memo": "off",
      "react-hooks/component-hook-factories": "off",
      "react-hooks/preserve-manual-memoization": "off",
      "react-hooks/immutability": "off",
      "react-hooks/globals": "off",
      "react-hooks/error-boundaries": "off",
      "react-hooks/set-state-in-render": "off",
      "react-hooks/config": "off",
      "react-hooks/gating": "off",
      "react-hooks/incompatible-library": "off",
      "react-hooks/unsupported-syntax": "off",

      // TypeScript strict rules
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": ["warn", {
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
        caughtErrorsIgnorePattern: "^_",
        ignoreRestSiblings: true,
      }],
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",
      "@typescript-eslint/no-non-null-assertion": "warn",
      "@typescript-eslint/prefer-nullish-coalescing": "warn",
      "@typescript-eslint/prefer-optional-chain": "warn",
      "@typescript-eslint/strict-boolean-expressions": "off",
      "@typescript-eslint/no-floating-promises": "warn",
      "@typescript-eslint/no-misused-promises": ["error", {
        checksVoidReturn: false,
      }],
      "@typescript-eslint/await-thenable": "error",
      "@typescript-eslint/no-unnecessary-condition": "off",
      "@typescript-eslint/no-unnecessary-type-assertion": "error",
      "@typescript-eslint/prefer-as-const": "error",
      "@typescript-eslint/consistent-type-imports": ["error", {
        prefer: "type-imports",
        fixStyle: "inline-type-imports",
      }],
      "@typescript-eslint/consistent-type-exports": "error",
      "@typescript-eslint/no-import-type-side-effects": "error",
      "@typescript-eslint/no-unsafe-assignment": "off",
      "@typescript-eslint/no-unsafe-member-access": "off",
      "@typescript-eslint/no-unsafe-call": "off",
      "@typescript-eslint/no-unsafe-argument": "off",
      "@typescript-eslint/no-unsafe-return": "off",
      "@typescript-eslint/no-confusing-void-expression": "off",
      "@typescript-eslint/no-deprecated": "warn",
      "@typescript-eslint/no-extraneous-class": "off",
      "@typescript-eslint/require-await": "off",
      "@typescript-eslint/restrict-template-expressions": "off",
      "@typescript-eslint/no-empty-function": "off",
      "@typescript-eslint/no-invalid-void-type": "off",
      "@typescript-eslint/no-unnecessary-type-conversion": "warn",
      "@typescript-eslint/no-redundant-type-constituents": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      "@typescript-eslint/no-require-imports": "off", // Allow require() for dynamic imports
      "@typescript-eslint/use-unknown-in-catch-callback-variable": "off",
      "@typescript-eslint/no-base-to-string": "off",
      "@typescript-eslint/no-non-null-asserted-optional-chain": "warn",
      "@typescript-eslint/no-unused-expressions": "off",
      "@typescript-eslint/restrict-plus-operands": "off",
      "@typescript-eslint/naming-convention": [
        "error",
        {
          selector: "variable",
          format: ["camelCase", "PascalCase", "UPPER_CASE"],
          leadingUnderscore: "allow",
        },
        {
          selector: "function",
          format: ["camelCase", "PascalCase"],
        },
        {
          selector: "typeLike",
          format: ["PascalCase"],
        },
        {
          selector: "interface",
          format: ["PascalCase"],
        },
        {
          selector: "enum",
          format: ["PascalCase"],
        },
        {
          selector: "enumMember",
          format: ["PascalCase", "UPPER_CASE"],
        },
      ],

      // General strict rules
      "no-console": ["warn", { allow: ["warn", "error", "log"] }], // Allow console.log for debugging
      "no-debugger": "error",
      "no-alert": "warn", // Allow confirm dialogs with warning
      "no-var": "error",
      "prefer-const": "error",
      "prefer-template": "error",
      "prefer-spread": "error",
      "prefer-rest-params": "error",
      "prefer-arrow-callback": "error",
      "arrow-body-style": ["error", "as-needed"],
      "no-param-reassign": ["warn", { props: false }],
      "no-nested-ternary": "warn",
      "no-unneeded-ternary": "error",
      "no-duplicate-imports": "off", // TypeScript handles type imports separately
      "object-shorthand": "error",
      "eqeqeq": ["error", "always", { "null": "ignore" }],
      "curly": ["error", "all"],
      "default-case": "error",
      "default-case-last": "error",
      "no-else-return": "error",
      "no-lonely-if": "off",
      "no-useless-return": "error",
      "no-useless-concat": "error",
      "no-useless-computed-key": "error",
      "no-useless-rename": "error",
      "no-throw-literal": "error",
      "prefer-promise-reject-errors": "error",
      "require-await": "off", // Allow async functions without await
      "no-return-await": "off", // Using @typescript-eslint/return-await instead
      "@typescript-eslint/return-await": ["error", "in-try-catch"],
      "max-lines-per-function": "off", // Components can be large
      "complexity": "off", // Complex functions can be necessary
      "max-depth": "off", // Nested logic can be necessary
      "max-params": ["warn", 5],
      "no-empty": "warn", // Allow empty blocks with warning
      "no-useless-escape": "warn", // Allow escape sequences in regex
      "no-empty-pattern": "off", // Allow empty destructuring patterns
      "no-case-declarations": "off", // Allow lexical declarations in case blocks
      "default-case": "off", // Not all switches need default
      "no-control-regex": "off", // Allow control characters in regex (terminal escape sequences)

      // Accessibility - some relaxed for pragmatism
      "jsx-a11y/click-events-have-key-events": "warn",
      "jsx-a11y/no-static-element-interactions": "warn",
      "jsx-a11y/anchor-is-valid": "warn",
      "jsx-a11y/no-autofocus": "off", // Allow autoFocus for better UX
      "jsx-a11y/heading-has-content": "off", // Allow empty headings for styling
      "jsx-a11y/label-has-associated-control": "warn",
      "jsx-a11y/no-noninteractive-element-interactions": "warn",
      "jsx-a11y/no-noninteractive-tabindex": "warn",
    },
  }
);
