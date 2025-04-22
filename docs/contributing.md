# Contributing Guide

Thank you for your interest in the DTVM_SolSDK project! This guide will help you understand how to effectively contribute to the project.

## Code of Conduct

All participants in the project must adhere to our code of conduct. Please read [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) for details.

## Ways to Contribute

You can contribute to DTVM_SolSDK in the following ways:

1. Report bugs and suggest features
2. Submit code changes
3. Improve documentation
4. Share use cases and feedback

## Development Environment Setup

Before starting to contribute code, please ensure you have set up your development environment according to the [Compilation Guide](compilation-guide.md).

## Reporting Bugs

If you discover an issue, please create an Issue on GitHub and provide the following information:

1. A brief description of the bug
2. Steps to reproduce
3. Expected behavior
4. Actual behavior
5. Environment information (operating system, Rust version, dependency versions, etc.)
6. If possible, provide relevant logs or screenshots

## Contributing Code

### Branch Strategy

- The `main` branch contains the latest stable version
- Feature development should use new branches created from the `main` branch
- Branch naming conventions:
  - Feature branches: `feature/description`
  - Bug fixes: `bugfix/description`
  - Documentation updates: `docs/description`

### Development Workflow

1. Fork the project repository to your own GitHub account
2. Clone the forked repository locally
   ```sh
   git clone https://github.com/<your-username>/DTVM_SolSDK.git
   cd DTVM_SolSDK
   ```
3. Add the main repository as a remote source
   ```sh
   git remote add upstream https://github.com/DTVMStack/DTVM_SolSDK.git
   ```
4. Create a new branch
   ```sh
   git checkout -b feature/your-feature-name
   ```
5. Implement your changes
6. Run tests to ensure code quality
   ```sh
   cargo test
   ```
7. Commit your changes
   ```sh
   git commit -m "Descriptive commit message"
   ```
8. Push to your fork
   ```sh
   git push origin feature/your-feature-name
   ```
9. Create a Pull Request (PR)

### Commit Message Guidelines

Please follow this commit message format:

```
<type>: <description>

[Optional detailed description]

[Optional issue closure: Fixes #123]
```

Types can be:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (not affecting code functionality)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Build process or auxiliary tool changes

### Code Review Process

After submitting a PR:
1. Maintainers will review your code
2. Changes or clarifications may be requested
3. Once the review is approved, the PR will be merged into the main branch

## Code Style Guide

We use the standard Rust code style, enforced by `rustfmt`:

```sh
# Format code
make fmt -f dev.makefile

# Run clippy for code quality checks
make fmt_check -f dev.makefile
```

Please ensure you run these commands before submitting code to ensure the code complies with the project style.

## Testing Guidelines

For new features or bug fixes, we encourage adding appropriate tests:

- Unit tests are located in the `src/tests/` directory
- Integration tests use example contracts in the `examples/` directory

Running tests:

```sh
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run integration test examples
cd examples/erc20
./test_simple_token.sh
```

## Documentation Contributions

Improving documentation is equally important:

1. Fix errors or inaccuracies in the documentation
2. Add missing information or examples
3. Improve clarity and usability of existing documentation

## Release Process

Project maintainers are responsible for version releases:

1. Version numbers follow [Semantic Versioning](https://semver.org/)
2. Each release version will update CHANGELOG.md
3. Release versions will be tagged and pushed to GitHub

## License

By contributing code, you agree that your contributions will be distributed under the project's Apache-2.0 license. Please see the [LICENSE](../LICENSE) file for details.

## Getting Help

If you need help during the contribution process, you can:

- Ask questions on GitHub Issues
- Check existing documentation
- Contact project maintainers

## Acknowledgements

We appreciate all contributors, regardless of the size of the contribution! 
