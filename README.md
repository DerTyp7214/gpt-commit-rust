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

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgements

This tool was built using the following libraries:

- [reqwest](https://crates.io/crates/reqwest)
- [colored](https://crates.io/crates/colored)
- [dotenv](https://crates.io/crates/dotenv)

Special thanks to the OpenAI team for their GPT-3 model.
