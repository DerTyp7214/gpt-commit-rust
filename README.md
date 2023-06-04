# gpt-commit-rust

**gpt-commit-rust** is a command-line tool that leverages GPT-3 to generate commit messages for your Git repository. It provides an interactive interface to compose and execute Git commands conveniently.

## Installation (source)

1. Make sure you have Rust and Cargo installed on your system.
2. Clone the repository: `git clone https://github.com/DerTyp7214/gpt-commit-rust.git`
3. Navigate to the project directory: `cd gpt-commit-rust`
4. Build the project: `cargo build --release`
5. The binary will be generated in the `target/release` directory.

## Installation (binary)

1. Download the latest release from the [releases page](https://github.com/DerTyp7214/gpt-commit-rust/releases)
2. Move the binary to a directory in your `PATH` environment variable
3. Make the binary executable: `chmod +x gpt-commit-rust`

## Usage

```shell
Usage: gpt-commit-rust [optional:option] [optional:files]

Options:
--help, -h: Shows the help message.
--version, -v: Shows the current version of the program.
--update, -u: Updates the program to the latest version.
--force-update, -f: Forces the update to the latest version.
--push, -p: Pushes the changes to the remote repository after running the commands.
--api-key: Sets the API key to use for GPT-3. You can also set the API key in the .env file.
--clear-api-key: Clears the API key from the config file.
```

## Getting Started

1. Run `gpt-commit-rust` in your Git repository's directory.
2. Use the interactive interface to compose your commit message.
3. Confirm the generated commands.
4. Optionally, use the `--push` option to push the changes to the remote repository.

## Examples

1. Generate commit commands without pushing changes:

   ```shell
   gpt-commit-rust
   ```

2. Generate commit commands and push changes:

   ```shell
   gpt-commit-rust --push
   ```

3. Set the GPT-3 API key:

   ```shell
   gpt-commit-rust --api-key YOUR_API_KEY
   ```

4. Clear the GPT-3 API key:

   ```shell
   gpt-commit-rust --clear-api-key
   ```

## Contribution

Thank you for your interest in contributing to **gpt-commit-rust**! Contributions are always welcome. To contribute, please follow these steps:

1. Fork the repository on GitHub.
2. Clone your forked repository to your local machine.
3. Create a new branch from the `dev` branch: `git checkout -b feature/my-new-feature`.
4. Make your desired changes to the codebase.
5. Test your changes to ensure they work as expected.
6. Commit your changes with descriptive commit messages.
7. Push your changes to your forked repository.
8. Create a pull request from your branch to the `dev` branch of the original repository.
9. Provide a clear and concise description of your changes in the pull request.
10. Be responsive to any feedback or questions related to your contribution.

Thank you for your contribution in advance!

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgements

This tool was built using the following libraries:

- [colored](https://crates.io/crates/colored)
- [dotenv](https://crates.io/crates/dotenv)
- [futures-util](https://crates.io/crates/futures-util)
- [git2](https://crates.io/crates/git2)
- [indicatif](https://crates.io/crates/indicatif)
- [lazy_static](https://crates.io/crates/lazy_static)
- [normpath](https://crates.io/crates/normpath)
- [os-version](https://crates.io/crates/os-version)
- [regex](https://crates.io/crates/regex)
- [reqwest](https://crates.io/crates/reqwest)
- [serde](https://crates.io/crates/serde)
- [serde_json](https://crates.io/crates/serde_json)
- [term_size](https://crates.io/crates/term_size)
- [tokio](https://crates.io/crates/tokio)
- [toml](https://crates.io/crates/toml)

Special thanks to the OpenAI team for their GPT-3 model.
