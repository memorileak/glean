# Glean API Documentation

## Technical Details

- Language: Rust
- Wire Protocol: JSON RPC
- Library: [jsonrpsee](https://github.com/paritytech/jsonrpsee)

## API Reference

### Repository Management

`repo_stats`

- Returns statistics for all repositories currently registered with Glean.
- Each entry includes the repository id, total file count, a status of `ok` or `error`, and an error message if the repository could not be walked successfully.

`add_repo`

- Registers a new repository with Glean, making it available for search and exploration.
- Requires a unique `id` and the absolute path to the repository on disk.
- The `id` must be a UTF-8 string. It serves as the stable identifier for the repository across all API calls (e.g. `mozilla-central` or `rust-lang/rust`).

`remove_repo`

- Unregisters a repository from Glean by its `id`.
- After removal, the repository will no longer appear in search results or be accessible via file operations.

### Searching

`search_file`

- Performs a fuzzy search over the file paths of all registered repositories using [`skim`](https://github.com/lotabout/skim).
- Accepts a query string and returns a ranked list of matching files, each with a `repo_id`, a `file_path` relative to the repository root, and a `score` reflecting the quality of the match.

`search_pattern`

- Searches for a structural code pattern across all files in all registered repositories using [`ast-grep`](https://ast-grep.github.io).
- Unlike file search, this is a precise structural match rather than a fuzzy one, so no score is returned.
- Returns a list of results grouped by file, each containing a `repo_id`, a `file_path` relative to the repository root, and a list of `Match` positions where the pattern was found.

### File Operations

`get_file_outline`

- Returns a structured outline of the symbols defined in a file, identified by `repo_id` and `file_path` (relative to the repository root).
- Symbol extraction is powered by [`tree-sitter`](https://tree-sitter.github.io) via `aerial.nvim`, providing accurate, language-aware results.
- Each `Symbol` includes its name, kind (e.g. `function`, `struct`, `class`), source range, and optionally nested child symbols.

`get_file_content`

- Returns the full source content of a file as a string.
- Requires the `repo_id` and `file_path` (relative to the repository root).

`get_matches_content`

- Resolves the source content for a set of positional matches within one or more files.
- Accepts a list of `MatchContentRequest` objects, each specifying a `repo_id`, `file_path`, and a list of `Match` ranges.
- Returns a corresponding list of `MatchContentResult` objects where each match is annotated with its actual source content.
- Intended to be called after `search_pattern` or `get_file_outline` to fetch the text at known positions without reading entire files.

### UI Configuration

`get_ui_config`

- Returns the current UI configuration as a key-value map (`UiConfig`).

`set_ui_config`

- Updates the UI configuration by merging a partial `UiConfig` map into the current configuration.
- Only the keys present in the provided map are updated; all other keys retain their existing values.

## Data Types

### Primitives

```typescript
// 1-based line and column position within a file
type Position = [line: number, col: number];
```

### Repository

```typescript
interface RepoStats {
  id: string;
  sequence: number;
  path: string; // absolute path on disk
  file_count: number;
  status: "ok" | "error";
  error?: string; // present when status is "error"
}
```

### Searching

```typescript
// A single positional match within a file
interface Match {
  start: Position;
  end: Position;
}

// Result item from search_file
interface FileSearchResult {
  repo_id: string;
  file_path: string; // relative to repository root
  score: number;
}

// Result item from search_pattern (per file)
interface PatternSearchResult {
  repo_id: string;
  file_path: string; // relative to repository root
  matches: Match[];
}
```

### File Operations

```typescript
type SymbolKind =
  | "function"
  | "method"
  | "struct"
  | "class"
  | "enum"
  | "interface"
  | "constant"
  | "field"
  | "module"
  | "type";

interface Symbol {
  name: string;
  kind: SymbolKind;
  start: Position;
  end: Position;
  children?: Symbol[]; // nested symbols, e.g. methods within a class
}

// Input item for get_matches_content
interface MatchContentRequest {
  repo_id: string;
  file_path: string; // relative to repository root
  matches: Match[];
}

// A match resolved with its source content
interface MatchWithContent {
  start: Position;
  end: Position;
  content: string;
}

// Result item from get_matches_content
interface MatchContentResult {
  repo_id: string;
  file_path: string;
  matches: MatchWithContent[];
}
```

### UI Configuration

```typescript
type UiConfig = Record<string, unknown>;
```
