# `repomix` (TS) vs `repoxide` (Rust) benchmark

## Setup

| Item | Value |
| --- | --- |
| Target repo | `https://github.com/yamadashy/repomix.git` |
| Target commit | `4c356f73251746c2cc3edcc68dbe19204aa1e950` |
| Target snapshot | `1072` files, `11.11 MiB` |
| TS tool | `repomix 1.13.1` on `Node v20.19.2` |
| Rust tool | `repoxide 0.1.0` |
| Host | `AMD EPYC 9634`, `6` visible vCPU, `11.68 GiB` RAM |
| Measured runs | `5` runs per tool after `1` warm-up run |

## Benchmark command

Both tools were run against the same local clone in default mode (not `--quiet`), with XML output pinned and extra non-core work disabled for parity:

```json
{
  "output": {
    "style": "xml",
    "git": {
      "sortByChanges": false
    }
  },
  "security": {
    "enableSecurityCheck": false
  }
}
```

```bash
repomix -c <config.json> -o repomix-bench.xml <local-target-repo>
repoxide -c <config.json> -o repoxide-bench.xml <local-target-repo>
```

Metrics were captured per process via `os.wait4`: `cpu time = user + sys`, `latency = wall clock`, `ram = max RSS`.

## Summary

| Tool | Output size | CPU time, mean | Latency, mean | Peak RAM, mean |
| --- | ---: | ---: | ---: | ---: |
| `repomix` (TS) | `4,344,566 B` | `6.393 s ± 0.149` | `2.213 s ± 0.047` | `444.9 MiB ± 2.8` |
| `repoxide` (Rust) | `4,377,520 B` | `1.434 s ± 0.035` | `0.856 s ± 0.020` | `78.0 MiB ± 0.2` |
| Advantage | `+0.76%` larger output for Rust | `4.46x` less CPU time | `2.59x` lower latency | `5.70x` lower RAM |

## Raw runs

| Run | `repomix` CPU | `repomix` latency | `repomix` RAM | `repoxide` CPU | `repoxide` latency | `repoxide` RAM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | `6.548 s` | `2.248 s` | `445.3 MiB` | `1.490 s` | `0.886 s` | `77.9 MiB` |
| 2 | `6.342 s` | `2.177 s` | `443.6 MiB` | `1.386 s` | `0.827 s` | `78.0 MiB` |
| 3 | `6.453 s` | `2.286 s` | `449.9 MiB` | `1.447 s` | `0.861 s` | `78.0 MiB` |
| 4 | `6.493 s` | `2.193 s` | `441.3 MiB` | `1.433 s` | `0.865 s` | `78.3 MiB` |
| 5 | `6.128 s` | `2.160 s` | `444.5 MiB` | `1.414 s` | `0.839 s` | `77.8 MiB` |

## Result

On this workload, `repoxide` was materially faster and lighter than the TypeScript `repomix` CLI while producing a similarly sized XML pack.

Note: `repoxide` currently packed `19` extra files from this repository because it honors `.ignore` and `.repoxideignore`, but not `.repomixignore` yet.
