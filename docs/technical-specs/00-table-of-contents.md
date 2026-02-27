# FS25 Save Editor - Technical Specifications

## Table of Contents

| # | Document | Description |
|---|----------|-------------|
| 01 | [Stack & Tooling](01-stack-and-tooling.md) | Technology choices, dependencies, development tools |
| 02 | [General Architecture](02-general-architecture.md) | Tauri architecture, frontend/backend communication, project structure |
| 03 | [Rust Backend](03-backend-rust.md) | Rust modules, XML parsing, data models, Tauri commands |
| 04 | [Vue Frontend](04-frontend-vue.md) | Vue structure, components, Pinia stores, routing, i18n |
| 05 | [Data Models](05-data-models.md) | TypeScript types and Rust structs, XML-to-model mapping |
| 06 | [Testing](06-tests.md) | Testing strategy, tools, coverage |

## Technical Decisions

| Decision | Choice |
|----------|--------|
| Desktop runtime | Tauri v2 |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript (Composition API) |
| UI Components | shadcn-vue (Reka UI + Tailwind CSS) |
| State management | Pinia |
| Internationalization | vue-i18n |
| Package manager | pnpm |
| Frontend tests | Vitest + Vue Test Utils |
| Backend tests | cargo test (native Rust) |
| XML parsing (Rust) | quick-xml + serde |
| Frontend bundler | Vite |
