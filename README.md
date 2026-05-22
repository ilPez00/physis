# Physis

Ontological mapper, PDCA dream engine, and Linguistic Routing Middleware.

Physis maps filesystem structure into an ontological trie, tracks goals through PDCA cycles, generates stochastic "dreams" for path exploration, and routes all systemic data through three linguistic filters for archival, logging, and semantic expansion.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│  Scanner     │────▶│  Ontology    │────▶│  PDCActor    │
│  (filesystem)│     │  Mapper      │     │  (Plan-Do-   │
└─────────────┘     └──────────────┘     │   Check-Act)  │
                                         └──────┬───────┘
                                                │
                          ┌─────────────────────┼──────────────┐
                          │                     │              │
                   ┌──────▼──────┐      ┌───────▼──────┐      │
                   │  DreamEngine │      │    Trie      │      │
                   │  (stochastic │      │  (Dynamic    │      │
                   │   mutation)  │      │  Vector)     │      │
                   └─────────────┘      └──────────────┘      │
                                                               │
                          ┌────────────────────────────────────▼──┐
                          │       Linguistic Router               │
                          │  ┌────────┬─────────┬──────────┐      │
                          │  │ Wenyan  │ Pirahã  │ Sanskrit │      │
                          │  │ (CJK    │ (silent │ (dreaming│      │
                          │  │  archive)│ logging)│ expansion)│    │
                          │  └────────┴─────────┴──────────┘      │
                          └───────────────────────────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| `scanner` | Filesystem scanning with hash-based change detection |
| `trie` | Dynamic Vector Trie for path-based token storage and retrieval |
| `mapper` | Ontology-aware filesystem-to-goal mapping |
| `config` | Ontology loading (human + machine domains) |
| `models` | Core types: Goal, Experience, Dream, OntologyEntry |
| `actor` | PDCA (Plan-Do-Check-Act) cycle engine |
| `dream` | Stochastic dream generation (mutation, graft, prune, cross-pollination) |
| `output` | Wiki, JSON graph, Mermaid, and domain report formatters |
| `network` | Watch directories for changes, network event detection |
| `mcp` | MCP server for external tool integration |
| `linguistic` | **Linguistic Routing Middleware** — Wenyan/Pirahã/Sanskrit filters |
| `cli` | Command-line interface via clap |

## Linguistic Routing Middleware

Three linguistic filters that intercept all data streams:

- **Wenyan** (Classical Chinese) — Compresses data into CJK path hierarchies for uberwiki archival. Maps 80+ English keywords to single characters (run→跑, body→身, code→碼).
- **Pirahã** (Caveman-Ultra) — Strips 160 function words for silent internal-only sub-process logging. Hidden from user output.
- **Sanskrit** (Dreaming) — Expands data through a 35-concept semantic net for generative background dreaming.

See [`src/linguistic/README.md`](src/linguistic/README.md) for full details.

## CLI Usage

```
physis scan <dir>                    # Scan directory, build ontological map
physis query <query>                 # Query the trie
physis dream [--count N]             # Generate stochastic dreams
physis evaluate <id> <grade>         # Evaluate a dream (0.0-1.0)
physis watch <dirs...>               # Watch directories for changes
physis stats                         # Show engine stats
physis config                        # Print current config
physis serve                         # Start MCP server (feature: mcp)
```

## Configuration

Default config loads built-in human (`praxis_ontology.json`) and machine (`machine_ontology.json`) ontologies. Custom ontologies can be provided via `PhysisConfig`.

```json
{
  "data_dir": "~/.physis",
  "dream_batch_size": 5,
  "pdca_stagnant_threshold": 0.2,
  "pdca_stagnant_window": 5
}
```

## Build

```bash
cargo build              # CLI only (default)
cargo build --features mcp     # With MCP server support
cargo build --features network # With directory watching
cargo build --features full    # All features
cargo test                     # Run test suite
```
