<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

This project is a Rust application that uses the `ignore` crate to traverse file systems, respecting .gitignore and other ignore files. Generate code that efficiently lists all traversed file paths to stdout.

Additionally, this project uses the `notify` crate to watch for cross-platform file system events, including renames. When a directory is renamed, traverse all child files of the newly named directory and report their new paths to stdout. Ensure that event handling is robust and that child paths are normalized for output and testing.

Event filtering: Access/Open events are completely filtered out and not reported. Only actual file modification, creation, removal, and rename events are reported to stdout.
