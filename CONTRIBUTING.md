# Contributing to Portico

Thank you for considering a contribution to **Portico** – we're thrilled to have your help!

---

## 1. Development Environment & Tooling

### Prerequisites

| Tool | Recommended Version | Purpose |
|------|---------------------|---------|
| **Node.js** | 18 LTS | SvelteKit front-end & tooling |
| **pnpm** | ≥ 8 | JS/TS package manager |
| **Rust** | stable (latest) | Tauri / engine / shared libraries |
| **Python** | 3.10 + | Bridge micro-service |
| **uv** | latest | ⚡ Ultra-fast Python package manager |
| **Docker & Compose** | latest | Local services & CI |
| **PostgreSQL** | 15 + | Primary database |
| **Atlas CLI** | latest | Database schema & ORM management |

### Clone & Install

```bash
# Fork then clone
$ git clone https://github.com/your-fork/Portico.git
$ cd Portico

# Install JS packages
$ cd app && pnpm install

# (Optional) Setup Python & Rust deps as needed

# Python deps with **uv** (recommended)
```bash
# From repo root
uv pip install -r server/bridge/pyproject.toml  # syncs exact versions
```

### Git Hooks with *pre-commit*

1. Install once:
   ```bash
   pipx install pre-commit   # or  pip install pre-commit
   ```
2. From the repo root:
   ```bash
   pre-commit install  # sets up `.git/hooks/pre-commit`
   ```
3. Hooks run automatically on `git commit` – they format code, lint, and run quick tests.
4. Update hooks at any time:
   ```bash
   pre-commit autoupdate
   ```

*(If you prefer a different environment manager, adapt accordingly.)*

### Commit Messages – Conventional Commits

We follow the [Conventional Commits](https://www.conventionalcommits.org/) spec so changelogs and releases can be automated.

```
<type>(optional-scope): <description>

[optional body]
[optional footer]
```

Common `<type>` values:
- **feat** – new feature
- **fix** – bug fix
- **docs** – documentation only changes
- **refactor** – code change that neither fixes a bug nor adds a feature
- **test** – adding or correcting tests
- **chore** – tooling & infra

### Pull-Request Checklist

Before opening a PR, please ensure:

- [ ] `pnpm test` / `cargo test` / `pytest` all pass
- [ ] `pre-commit run --all-files` succeeds
- [ ] New/changed code is fully typed and documented
- [ ] Screenshots or GIFs for UI changes are attached
- [ ] Corresponding docs & READMEs are updated
- [ ] PR description explains **why** the change is needed

CI will verify most of the above automatically.

## 2. Community Guidelines

Our goal is a welcoming and collaborative environment. By participating you agree to abide by our **Code of Conduct**:

1. **Be kind & respectful.** Harassment or discrimination of any form will not be tolerated.
2. **Assume positive intent.** We all make mistakes – focus on learning and building together.
3. **Give constructive feedback.** Offer actionable suggestions; celebrate each other's successes.
4. **Keep discussions public when possible.** This helps shared learning and avoids knowledge silos.
5. **Respect review boundaries.** Maintainers may request changes or defer large-scale refactors to future work.

Violations may result in review dismissal or, in severe cases, revocation of contribution privileges.

## 3. Professional & Legal Notes

📜 **License.** Portico is released under the Business Source License 1.1 (BSL 1.1). By contributing you agree your work will be distributed under the project's license. The license converts to Apache 2.0 on **July 1 2030**.

⚖️ **Liability.** Contributions are provided **as-is**; the project and its contributors make no warranties. You are responsible for ensuring you have the right to submit any code, data, or documentation you contribute.

🔒 **Security.** If you discover a security vulnerability, please **do not** open a public issue. Instead email **security@stoamedical.com**.

🔄 **CLA.** For substantial contributions we may ask you to sign a Contributor License Agreement to affirm the above.

---

### Need Help?

• **Docs:** check the various `README.md` files throughout the repo.
• **Discussions / Questions:** open a [GitHub Discussion](https://github.com/Stoa-Medical/Portico/discussions) or reach out on our community Slack.
• **Bugs:** open a descriptive [Issue](https://github.com/Stoa-Medical/Portico/issues) with reproduction steps.

We appreciate your time and effort – **thank you for making Portico better!** :tada:
