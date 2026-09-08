# Contributing

Choose a workflow for the part you change. The Rust core is at the repository root, language packages are under `bindings/`, and the embedded UI is under `console/`. Do not commit generated messages or web assets.

## Documentation only

```bash
python3 scripts/check_docs.py
# or: just check-docs
```

This uses only Python and Git. It checks local Markdown file links and English/Chinese page pairs, including new untracked files. It does not check external URLs or heading anchors.

Update `docs/en/` and `docs/zh/` together. For a new user-facing page, add its slug to `console/scripts/bundle-docs.mjs` and link it from an existing entry point.

The marked Python blocks in both READMEs and Python API guides are executable examples, checked directly in CI. Keep each complete and give its expected output. After changing them, run the native SDK setup below, then:

```bash
python scripts/test_doc_examples.py
# or: just test-doc-examples
```

The test uses an isolated broker for each example, selects available loopback ports, and runs all 12 blocks in separate processes.

## Python and Rust setup

Use the repository's CI as the versioned build reference: Rust stable with edition 2024 support, Python **3.12** for the CI baseline, protobuf compiler **35.1**, native C/C++ build tools and ZeroMQ build dependencies. Default builds also need Node.js **22**, pnpm **11** and the console assets. `just` is the task runner. See [CI](.github/workflows/ci.yml) for exact package installation steps and [build profiles](docs/en/build-profiles.md) for builds without web assets. Release-package users do not need to generate messages themselves.

Run from a checkout's root:

```bash
python3.12 -m venv .venv
source .venv/bin/activate
python -m pip install maturin 'protobuf>=7.35,<8'
protoc --version  # libprotoc 35.1
just python-dev
python scripts/test_doc_examples.py
```

`just python-dev` generates Python/Rust messages, builds web assets if absent, and installs the current native SDK into the active virtual environment. An older installed wheel does not validate the current checkout. If console sources or bundled documentation changed, run `just console` explicitly before rebuilding. On Windows use the virtual environment activation command for your shell; the `just` recipes themselves require Bash.

## Checks by change

| Change | Checks |
| --- | --- |
| Rust core | `just test-rust`; feature boundaries: `just test-rust-minimal` |
| Python wrappers/native API | `just test-python` + `just test-python-native` |
| TypeScript | `just ts-dev` + `just test-typescript` |
| C++ | `just cpp-dev` + `just test-cpp` |
| Java | `just java-dev` + `just test-java` |
| Android | `just android-dev` + `just test-android` (SDK/NDK required) |
| Cross-language protocol | `just test-interop` (all participating toolchains required) |
| Console | `just console`; see [console development](console/README.md) |
| ROS bridge | `just check-ros2-shim` for Rust type checks, plus real ROS runs for the changed language/routes |

Host tests and shim checks do not replace device or real ROS verification. Follow [support status](docs/en/support.md), and attach environment plus commands when adding runtime claims. Performance runs are separate and overwrite generated reports; see [performance](docs/en/performance.md).

Before submitting, explain the user-visible change, checks run and remaining limitations. Add regression coverage for behavior changes; document compatibility changes alongside code.
