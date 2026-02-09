# life-os

Personal system checker and organizer for macOS. `life-os` validates a folder layout from a JSON spec, creates missing folders, and tidies the Desktop and Downloads safely by default.

**Features**

- Validate required folders from a spec (`doctor`).
- Create missing folders from the same spec (`init`).
- Tidy Desktop screenshots and Downloads with a dry-run default (`tidy`).
- Colorful output with a `--plain` mode for scripts.

**Build And Run**

```bash
cargo build

cargo run -- doctor
cargo run -- doctor --verbose

cargo run -- init --verbose

cargo run -- tidy
cargo run -- tidy --apply
cargo run -- tidy --apply --all
```

**Install And Add To PATH**

```bash
cd /Users/rasmus/System/life-os/repo
cargo install --path .
```

Ensure `~/.cargo/bin` is on your PATH (Zsh):

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Verify:

```bash
which life-os
life-os --help
```

**Commands**

- `doctor` checks the required folder layout. Exit code `0` when satisfied, `1` when missing folders exist.
- `init` creates missing folders from the spec.
- `tidy` reports Desktop/Downloads status and planned actions. It only moves/deletes files when `--apply` is set.

`tidy` behavior:

- Desktop: moves macOS screenshot files (`Screenshot *.png`) to `~/Documents/screenshots`.
- Downloads: marks items older than 7 days for deletion, or everything when `--all` is set.

**Configuration**
The spec file is loaded from:

- `~/System/life-os/config/spec.json`

The spec format is:

```json
{
	"version": 1,
	"areas": [
		{
			"name": "System",
			"root": "~/System",
			"required": [
				{ "path": "apps" },
				{
					"path": "life-os",
					"children": [{ "path": "repo" }, { "path": "config" }]
				}
			]
		}
	]
}
```

Notes:

- `root` supports `~/` and is expanded against your home directory.
- `required` supports nested `children` for deeper trees.

**Development**

```bash
cargo test
```
