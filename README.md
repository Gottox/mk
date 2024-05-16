# mk
`mk` is a command line tool that analyzes the current working dir, tries to find the current build system and runs it.

## Usage

`mk` is designed to work without configuration. Just `cd` to your project directory and type `mk`. If the build system is supported, it will be detected and run.

## Supported build systems

- `make`
- `cmake`
- `cargo`
- `meson`

## `Mk.yaml`

If you further need to configure `mk`, you can create a `.Mk.yaml` file in your project directory. The following options are supported:

- `build_system`: The name of the build system to use. `mk` tries to autodetect it.
- `default`: A list of arguments that are passed to the build system if you don't provide any.
- `configure`: A list of arguments that are passed to the configure step of the build system. Not all build systems support this.
- `build_dir`: The directory relative to the project root where the build is configured. Not all build systems support this. Default is `build`.

Example:

```yaml
build_system: cmake
default:
  - test
configure:
  - -DCMAKE_BUILD_TYPE=Release
build_dir: /tmp
```
