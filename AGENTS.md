# AGENTS.md — PeaceWave

Rust backend service for the [Peace CLI](https://github.com/sohanemon/peace).

Responsibilities (planned): Quran wallpapers and metadata, categories/tags/collections, random wallpaper discovery, image uploads and processing, Cloudflare R2 storage, REST API for the Peace CLI and other clients.

## Technology stack

- Rust (stable, edition 2024)
- Axum, Tokio, Tower / tower-http
- Serde
- PostgreSQL + SQLx
- Cloudflare R2 (S3-compatible API)
- Utoipa (OpenAPI)
- tracing

**Do not assume every dependency is installed or every feature exists.** Introduce dependencies only when the corresponding feature is actually implemented. Current dependencies: `axum`, `tokio` (`macros`, `rt-multi-thread`, `signal`), `serde_json`, `dotenvy`.

## 1. Architecture principles

- Modular monolith. No separate Cargo crate per module.
- `main.rs` stays tiny: bootstrap and start the server only.
- Separate HTTP handlers, business logic, data access, configuration, infrastructure, and domain models.
- HTTP handlers must not contain business logic or database queries.
- Business logic must not depend on Axum.
- Database access is isolated in repository/data-access modules.
- External services (Cloudflare R2) are isolated behind dedicated modules.
- Share state through an explicit application state type.
- Dependency injection via constructors or app state — no global mutable state.
- Clear module boundaries over unnecessary abstractions.

## 2. Target project structure

Target architecture — **not** scaffolding to create now. Add modules only when needed. Avoid empty modules and unnecessary files.

```text
peace-wave/
├── AGENTS.md
├── Cargo.toml
├── Cargo.lock
├── .env.example
├── .gitignore
├── README.md
├── migrations/
├── docs/
└── src/
    ├── main.rs
    ├── lib.rs
    ├── app.rs
    ├── config.rs
    ├── state.rs
    ├── error.rs
    ├── routes.rs
    ├── routes/
    │   ├── health.rs
    │   └── wallpapers.rs
    ├── wallpaper.rs
    ├── wallpaper/
    │   ├── model.rs
    │   ├── service.rs
    │   └── repository.rs
    ├── category.rs
    ├── category/
    │   ├── model.rs
    │   ├── service.rs
    │   └── repository.rs
    ├── storage.rs
    ├── storage/
    │   └── r2.rs
    ├── infrastructure.rs
    └── infrastructure/
        └── database.rs
```

### Module declaration style

**Rust 2018+ layout — no `mod.rs` files.** `mod.rs` is pre-2018 style; avoid it.

- Module file shares the module name: `config.rs`, not `config/mod.rs`.
- Submodules live in a same-named sibling directory: `routes.rs` declares `pub mod health;` with the file at `routes/health.rs`.
- `mod` declarations go at the top of the parent module file.
- A domain with no submodules is a single file (`state.rs`); add the sibling directory only when the module actually grows submodules. A leaf `.rs` file (e.g. `category.rs`) is preferred over `category/mod.rs` until subdividing is justified.

## 3. File and module responsibilities

- `main.rs` — entry point, server startup only.
- `lib.rs` — library root, public module declarations.
- `app.rs` — router construction and route composition.
- `config.rs` — environment-based application configuration.
- `state.rs` — shared application state and dependencies.
- `error.rs` — application errors + HTTP error conversion.
- `routes.rs` + `routes/` — HTTP routing root and per-resource handlers (`health.rs`, `wallpapers.rs`).
- Domain modules (`wallpaper.rs` + `wallpaper/`, `category.rs` + `category/`) — `model.rs`, `service.rs`, `repository.rs` alongside a module root that declares its submodules.
- `storage.rs` + `storage/` — object storage integrations (Cloudflare R2: `r2.rs`).
- `infrastructure.rs` + `infrastructure/` — external infrastructure (database connections: `database.rs`).
- `migrations/` — schema migrations.
- `docs/` — architecture decisions and technical documentation.

Do not put unrelated functionality into a generic `utils` module.

## 4. Coding conventions

- Idiomatic modern Rust.
- `snake_case` for modules, functions, variables; `PascalCase` for types and traits.
- `Result` and `?` for error propagation.
- No `unwrap()` / `expect()` in production code.
- `thiserror` for structured application errors when appropriate.
- `tracing` for application logging (not `println!` in production paths).
- Functions focused and reasonably small.
- Composition over abstraction; traits only when they provide real boundaries, testability, or interchangeable implementations.
- Avoid unnecessary cloning and shared mutable state.
- Keep public interfaces small.
- Follow `cargo fmt` and `cargo clippy`.

## 5. HTTP and API conventions

- RESTful, resource-oriented endpoints.
- Public API routes versioned under `/v1`.
- Keep HTTP request/response types separate from internal domain models when appropriate.
- Consistent JSON error responses.
- Validate incoming user input.
- Appropriate HTTP status codes.
- Never expose internal errors, secrets, or database details to API clients.

## 6. Database and storage conventions

For future implementation:

- SQLx for PostgreSQL.
- SQL queries and database access stay out of HTTP handlers.
- Schema changes via migrations.
- **Never modify an applied migration.**
- Wallpaper binaries live in Cloudflare R2, not PostgreSQL.
- PostgreSQL stores object keys and relevant metadata.
- Public images delivered directly via R2/CDN URLs — do not proxy image bytes through Axum.
- Never hardcode credentials or secrets.

## 7. Development workflow

**Develop incrementally, one task at a time.**

- Implement only what the current request asks for.
- No preemptive implementation of roadmap features.
- Before adding a new module — check it is actually needed.
- Before adding a dependency — explain why it is necessary.
- Do not restructure unrelated parts of the codebase.
- Preserve existing working functionality.
- Update this document when permanent architectural decisions change.
- If a request conflicts with these rules, explain the conflict before proceeding.

## 8. Definition of done

- Code compiles.
- Relevant tests run and pass.
- `cargo fmt --check` clean.
- `cargo clippy -- -D warnings` clean when applicable.
- Report: files changed, dependencies added, verification results.
- Never claim tests passed if they were not executed.
