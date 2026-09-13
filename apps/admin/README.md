# Foundry Admin Dashboard

Foundry Visual Admin Dashboard SPA (Single Page Application) built with React, TypeScript, Vite, and Tailwind CSS.

---

## 🛠️ Tech Stack

- **Framework**: React 18
- **Language**: TypeScript 5.7+ (Project References & Strict Type Safety)
- **Build Tool**: Vite 6
- **Styling**: Tailwind CSS 3.4 & PostCSS
- **Icons**: Lucide React
- **Internationalization**: i18next & react-i18next (en-US, zh-CN)
- **Code Standards & Formatting**: Prettier 3 + `prettier-plugin-tailwindcss`
- **Linting**: ESLint 10+ Flat Config (`eslint.config.js`) + `eslint-config-prettier`

---

## 🚀 Quick Start

### 1. Install Dependencies

```bash
pnpm install
```

### 2. Start Development Server

```bash
pnpm dev
```

Development server will run at `http://localhost:3000` (or `http://localhost:5173`) with API proxy targeting `http://localhost:8080`.

### 3. Build for Production

```bash
pnpm build
```

The output bundle will be generated in `dist/`.

### 4. Preview Production Build

```bash
pnpm preview
```

---

## 🔍 Code Standards & Quality Assurance

This project adheres to modern frontend engineering best practices with a 3-pillar quality assurance pipeline:

| Command                                      | Tool                  | Purpose                                                                   |
| :------------------------------------------- | :-------------------- | :------------------------------------------------------------------------ |
| `pnpm typecheck` (or `pnpm type-check`)      | TypeScript (`tsc -b`) | Performs comprehensive static type checking without emitting files        |
| `pnpm lint`                                  | ESLint                | Checks code style, React hooks rules, and TypeScript best practices       |
| `pnpm lint:fix`                              | ESLint                | Automatically fixes fixable linting issues                                |
| `pnpm format:check` (or `pnpm format-check`) | Prettier              | Checks if all files match formatting standards and Tailwind class sorting |
| `pnpm format`                                | Prettier              | Formats code, CSS, JSON, and sorts Tailwind CSS classes                   |
| `pnpm check`                                 | All                   | Executes `typecheck` + `lint` + `format:check` in sequence                |

### Toolchain Integration Highlights

1. **Prettier 3 + Tailwind CSS Plugin (`.prettierrc`)**:
   - Standardizes indentation (2 spaces), single quotes, trailing commas, and line length (100).
   - `prettier-plugin-tailwindcss` automatically organizes Tailwind utility class names in a consistent, predictable order.
2. **ESLint 9+ Flat Config (`eslint.config.js`) + `eslint-config-prettier`**:
   - Eliminates formatting rule conflicts between ESLint and Prettier.
   - Enforces React Hooks rules and TypeScript best practices.
3. **TypeScript Project References (`tsconfig.app.json` & `tsconfig.node.json`)**:
   - Isolates application code and Node configuration files (e.g. `vite.config.ts`).
   - Caches build metadata in `node_modules/.tmp/` to avoid repository clutter.
4. **EditorConfig (`.editorconfig`)**:
   - Synchronizes indentation, character encoding, and newline behavior across various IDEs (VS Code, JetBrains, etc.).
5. **CI Automation (`.github/workflows/ci.yml`)**:
   - Automatically runs `pnpm format:check`, `pnpm lint`, `pnpm typecheck`, and `pnpm build` on push and pull requests.

---

## 📁 Directory Structure

```
apps/admin/
├── src/
│   ├── components/       # Reusable UI widgets and layout shell
│   ├── locales/          # i18n configurations and translation dictionaries
│   ├── pages/            # Page-level route views (Dashboard, Systems, Models, Configs, etc.)
│   ├── services/         # API client & backend service integration
│   ├── types/            # TypeScript type definitions and interfaces
│   ├── utils/            # Helper utilities (cn, theme, etc.)
│   ├── App.tsx           # Application root component & routing state
│   ├── index.css         # Global Tailwind styles
│   └── main.tsx          # React application entry point
├── eslint.config.js      # Modern ESLint flat configuration
├── .prettierrc           # Prettier configuration with Tailwind plugin
├── .prettierignore       # Prettier ignore rules
├── tsconfig.json         # TypeScript project references root
├── tsconfig.app.json     # TypeScript application config
├── tsconfig.node.json    # TypeScript Vite/Node config
├── vite.config.ts        # Vite configuration & proxy settings
└── tailwind.config.js    # Tailwind CSS design system config
```
