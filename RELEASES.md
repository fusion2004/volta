# Version 0.6.3

- `volta install` will no longer error when installing a scoped binary package (#537)

# Version 0.6.2

- Added `volta list` command for inspecting the available tools and versions (#461)

# Version 0.6.1

- Windows users will see a spinner instead of a � when Volta is loading data (#511)
- Interrupting a tool with Ctrl+C will correctly wait for the tool to exit (#513)

# Version 0.6.0

- Allow installing 3rd-party binaries from private registries (#469)

# Version 0.5.7

- Prevent corrupting local cache by downloading tools to temp directory (#498)

# Version 0.5.6

- Improve expected behavior with Yarn in projects (#470)
- Suppress an erroneous "toolchain" key warning message (#486)

# Version 0.5.5

- Proper support for relative paths in Bin hooks (#468)
- Diagnostic messages for shims with `VOLTA_LOGLEVEL=debug` (#466)
- Preserve user order for multiple tool installs (#479)

# Version 0.5.4

- Show additional diagnostic messages when run with `--verbose` (#455)

# Version 0.5.3

- Prevent unnecessary warning output when not running interactively (#451)
- Fix a bug in load script for fish shell on Linux (#456)
- Improve wrapping behavior for warning messages (#453)

# Version 0.5.2

- Improve error messages when running a project-local binary fails (#426)
- Fix execution of user binaries on Windows (#445)

# Version 0.5.1

- Add per-project hooks configuration in `<PROJECT_ROOT>/.volta/hooks.json` (#411)
- Support backwards compatibility with `toolchain` key in `package.json` (#434)

# Version 0.5.0

- Rename to Volta: The JavaScript Launcher ⚡️
- Change `package.json` key to `volta` from `toolchain` (#413)
- Update `volta completions` behavior to be more usable (#416)
- Improve `volta which` to correctly find user tools (#419)
- Remove unneeded lookups of `package.json` files (#420)
- Cleanup of error messages and extraneous output (#421, #422)

# Version 0.4.1

- Allow tool executions to pass through to the system if no Notion platform exists (#372)
- Improve installer support for varied Linux distros

# Version 0.4.0

- Update `notion install` to use `tool@version` formatting for specifying a tool (#383, #403)
- Further error message improvements (#344, #395, #399, #400)
- Clean up bugs around installing and running packages (#368, #390, #394, #396)
- Include success messages when running `notion install` and `notion pin` (#397)

# Version 0.3.0

- Support `lts` pseudo-version for Node (#331)
- Error message improvements
- Add `notion install` and `notion uninstall` for package binaries
- Remove autoshimming

# Version 0.2.2

- Add `notion which` command (#293)
- Show progress when fetching Notion installer (#279)
- Improved styling for usage information (#283)
- Support for `fish` shell (#266, #290)
- Consolidate binaries, for a ~2/3 size reduction of Notion installer (#274)

# Version 0.2.1

- Move preventing globals behind a feature flag (#273)

# Version 0.2.0

- Add support for OpenSSL 1.1.1 (#267)
- Fix: ensure temp files are on the same volume (#257)
- Intercept global package installations (#248)
- Fix: make npx compatible with prelrease versions of npm (#239)
- Fix: make `notion deactivate` work infallibly, without loading any files (#237)
- Fix: make `"npm"` key optional in `package.json` (#233)
- Fix: publish latest Notion version via self-hosted endpoint (#230)
- Fix: eliminate excessive fetching and scanning for exact versions (#227)
- Rename `notion use` to `notion pin` (#226)
- Base filesystem isolation on `NOTION_HOME` env var (#224)
- Fix: robust progress bar logic (#221)
- Use JSON for internal state files (#220)
- Support for npm and npx (#205)
- Changes to directory layout (#181)

# Version 0.1.5

- Autoshimming! (#163)
- `notion deactivate` also unsets `NOTION_HOME` (#195)
- Implemented `notion activate` (#201)
- Fix for Yarn over-fetching bug (#203)

# Version 0.1.4

- Fix for `package.json` parsing bug (#156)

# Version 0.1.3

- Fix for Yarn path bug (#153)

# Version 0.1.2

- Correct logic for computing `latest` version of Node (#144)
- Don't crash if cache dir was deleted (#138)
- Improved tests (#135)

# Version 0.1.1

- Support for specifying `latest` as a version specifier (#133)
- Suppress scary-looking symlink warnings on reinstall (#132)
- Clearer error message for not-yet-implemented `notion install somebin` (#131)
- Support optional `v` prefix to version specifiers (#130)

# Version 0.1.0

First pre-release, supporting:

- macOS and Linux (bash-only)
- `notion install` (Node and Yarn only, no package binaries)
- `notion use`
- Proof-of-concept plugin API
