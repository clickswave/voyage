# Contributing to Voyage

Thank you for considering contributing to Voyage! We appreciate your help in making this advanced subdomain enumeration tool even better. Please take a moment to review these guidelines before contributing.

## Getting Started

1. **Fork the Repository**: Start by forking the repository to your GitHub account.
2. **Clone Your Fork**: Clone your fork to your local machine using:
   ```sh
   git clone https://github.com/your-username/voyage.git
   ```
3. **Set Upstream**: Add the upstream repository to keep your fork updated:
   ```sh
   git remote add upstream https://github.com/clickswave/voyage.git
   ```
4. **Create a Branch**: Work on a separate branch for your contribution:
   ```sh
   git checkout -b feature-or-bugfix-name
   ```
5. **Install Dependencies**: Ensure you have Rust installed. Then, navigate to the project directory and run:
   ```sh
   cargo build
   ```

## Code Guidelines

- **Follow Rust Best Practices**: Use idiomatic Rust and follow Clippy suggestions.
- **Keep Code Readable**: Write clean and well-documented code.
- **Ensure Thread-Safety**: Voyage is multi-threaded; handle concurrency carefully.
- **Error Handling**: Use proper error handling (`Result` and `Option` where appropriate).
- **Database Changes**: If modifying SQLite schema, double-check compatibility.

## Contribution Types

You can contribute in various ways, including:

- **Bug Fixes**: Identify and fix bugs.
- **Feature Additions**: Propose and add new features.
- **Documentation Improvements**: Enhance documentation.
- **Performance Optimizations**: Improve speed and resource usage.

## Submitting a Pull Request (PR)

1. **Commit Changes**: Follow clear commit messages:
   ```sh
   git commit -m "Fix: Resolved issue #123 by handling null values"
   ```
2. **Push to Your Fork**:
   ```sh
   git push origin feature-or-bugfix-name
   ```
3. **Open a PR**: Go to the original repository and open a pull request.
4. **PR Review Process**:
    - PRs will be reviewed for correctness, efficiency, and security.
    - Maintain a respectful and constructive discussion.

## Reporting Issues

- **Check for Duplicates**: Ensure the issue hasn’t already been reported.
- **Provide Details**: Include OS, Rust version, steps to reproduce, and expected behavior.
- **Label Appropriately**: Use labels like `bug`, `enhancement`, or `documentation`.

## Code of Conduct

Voyage follows a **zero-tolerance** policy for harassment or discrimination. Be respectful in discussions.

---

We appreciate your contributions to Voyage! 🚀
