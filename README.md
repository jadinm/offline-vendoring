# offline-vendoring

## Getting Started

This project is managed by `cargo`:

- Use `cargo build` to build the package.
- Use `cargo nextest run` to launch the series of tests (requires the [development dependencies](#setting-up-dev-environment) to work).

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
