# Commit Convention

To maintain consistency and readability in the codebase, all commits to this project should follow the convention below. These conventions are based on [Conventional Commits](https://www.conventionalcommits.org/).

## Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

### Type

Must be one of the following:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing or correcting existing tests
- **build**: Changes that affect the build system or external dependencies (example: cargo, github actions)
- **ci**: Changes to CI configuration files and scripts
- **chore**: Other changes that don't modify src or test files

### Scope

The scope should specify the area of the change, for example:

- **core**: Core library code
- **stdlib**: Standard library
- **examples**: Example code
- **docs**: Documentation related
- **tools**: Tool related
- **deps**: Dependency related
- **ci**: CI related
- **test**: Test related
- **other**: Other

### Description

- Use the imperative, present tense: "change" not "changed" or "changes"
- Don't capitalize the first letter
- No period at the end

### Body

- Use the imperative, present tense
- Should include the motivation for the change and contrast this with previous behavior

### Footer

- **BREAKING CHANGE**: Should start with "BREAKING CHANGE:" followed by a space or two newlines, then a description of the change
- References to issues: Can reference related issues, e.g., "Closes #123, #456"

## Examples

```
feat(core): add WebAssembly export function support

Add support for WebAssembly export functions, making it easier for developers to interact with external systems.

BREAKING CHANGE: Export functions now need to be explicitly marked, previously implicit exports are no longer supported.
Closes #123
```

```
fix(stdlib): fix heap overflow in memory allocator

Fix heap overflow that could occur when allocating memory too large.

Closes #456
```

```
docs: update WebAssembly interoperability documentation
```

## Automatic Validation

All commits will be automatically validated using GitHub Actions to ensure they comply with the above conventions. Commits that do not comply with the convention may be automatically rejected. 