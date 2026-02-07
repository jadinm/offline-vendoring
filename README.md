# Offline vendoring

The goal of that project is to package different external resources for offline install in air-gaped environments.

The following resources can be packaged:

1. Rust tools (e.g., cargo-audit)
2. Rust crates
3. Python packages (e.g., pre-commit)
4. Git repositories (e.g., a pre-commit hook repository or the rustsec/advisory-db for cargo-audit to work offline)
5. List of arbitrary paths along with a customizable install command (e.g., a list of vscode extensions)

We expect python, pip, rust, and git to be already installed in the offline machine.

## Getting Started

This project is managed by `cargo`:

- Use `cargo build` to build the package.
- Use `cargo nextest run` to launch the series of tests.

The compilation produces two cargo subcommands:
- `cargo-offline-package` downloads external resources and generates a .tar.gz archive.
- `cargo-offline-install` unpacks the archive and install all the resources in the offline machine.

Feel free to use "--help" on each binary to get the whole list of CLI arguments.

## Installing

On the online machine, you can use:

```rust
cargo install --path .
```

For the offline machine, copy the `${CARGO_HOME}/bin/cargo-offline-install`
(or in `${HOME}/bin/cargo-offline-install` if `CARGO_HOME` is unset)
binary from the online machine to the offline machine.

### Requirements

- A python setup with pip on both online and offline machines.
- A rust setup on both online and offline machines.
- Git for git mirrors on both online and offline machines.
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) to download rust tools faster on the online machine.

### Packaging external resources

Those steps need to be run in a machine with direct or indirect access to internet.

1. If you need rust crates offline, copy each Cargo.toml of all your rust packages and workspaces.
2. If you need python dependencies, create one or more requirement.txt files, one by python project.
3. If you need git mirroring, create in advance in the offline environment a repo to push the mirror to.
4. Create a configuration file. You can check a complete example at [example_settings.yaml](./example_settings.yaml).
5. Though not required, all the previously cited files should be versioned somewhere for convenience's sake.
6. Considering the example configuration, run the following command:

    ```shell
    RUST_LOG=info cargo offline-package ./example_settings
    ```

    Change the setting file to your own file.
    The absence of the file extension in the command argument is on purpose.
    We support multiple formats.
    The program will lookup for any file with a file extension corresponding to one of the following formats: JSON, TOML, YAML, INI, RON.

    The archive will be generated in the working directory.

### Installing external resources on the offline machine

1. Import the generated archive (and the `cargo-offline-install` binary if needed) in the offline machine.
2. Run the following command in the location where you wish the external resources to be downloaded:

    ```shell
    RUST_LOG=info cargo offline-install /path/to/the/generated/archive
    ```

## Setting Up Dev Environment

### Pre-commit Installation

This repository uses [`pre-commit`](https://pre-commit.com/) to check code.
This requires an additional manuel step.

There are two possible ways of enabling pre-commit, either only for this repository or user-wide.

If you go for a per-repositoy install, run here:

```shell
pre-commit install
```

If you think that running `pre-commit install` on each of your projects is tedious, there is an alterantive.
You can enable the pre-commit hooks user-wide.

For that, we use [Git Template Directory](https://git-scm.com/docs/git-init#_template_directory) that stores files to be copied over any newly cloned or initilized repository.
You need to choose a folder that will hold these scripts:

```shell
git config --global init.templateDir /path/to/chosen/template/directory
```

Then, we ask pre-commit to generate a pre-commit hook for that template directory:

```shell
pre-commit init-templatedir /path/to/chosen/template/directory
```

From now on, every newly cloned or initialized repository will have a hook bash scripts `.git/hooks/pre-commit`.
This script executes pre-commit software if any hook is configured in a `.pre-commit-config.yaml`.
If there is no hook defined, the commits are always accepted.
