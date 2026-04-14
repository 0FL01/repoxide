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

Both tools were run against the same local clone, with XML output pinned and extra non-core work disabled for parity:

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
repomix --quiet -c <config.json> -o repomix-bench.xml <local-target-repo>
repoxide --quiet -c <config.json> -o repoxide-bench.xml <local-target-repo>
```

Metrics were captured per process via `os.wait4`: `cpu time = user + sys`, `latency = wall clock`, `ram = max RSS`.

## Summary

| Tool | Output size | CPU time, mean | Latency, mean | Peak RAM, mean |
| --- | ---: | ---: | ---: | ---: |
| `repomix` (TS) | `4,344,566 B` | `6.628 s ± 0.107` | `2.232 s ± 0.077` | `444.7 MiB ± 9.5` |
| `repoxide` (Rust) | `4,377,520 B` | `0.175 s ± 0.005` | `0.057 s ± 0.004` | `14.9 MiB ± 0.2` |
| Advantage | `+0.76%` larger output for Rust | `37.95x` less CPU time | `39.10x` lower latency | `29.92x` lower RAM |

## Raw runs

| Run | `repomix` CPU | `repomix` latency | `repomix` RAM | `repoxide` CPU | `repoxide` latency | `repoxide` RAM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | `6.437 s` | `2.122 s` | `447.7 MiB` | `0.179 s` | `0.063 s` | `14.8 MiB` |
| 2 | `6.659 s` | `2.248 s` | `454.6 MiB` | `0.165 s` | `0.060 s` | `14.6 MiB` |
| 3 | `6.596 s` | `2.189 s` | `445.0 MiB` | `0.173 s` | `0.052 s` | `15.1 MiB` |
| 4 | `6.709 s` | `2.245 s` | `426.7 MiB` | `0.178 s` | `0.055 s` | `15.0 MiB` |
| 5 | `6.737 s` | `2.355 s` | `449.6 MiB` | `0.178 s` | `0.055 s` | `14.8 MiB` |

## Result

On this workload, `repoxide` was materially faster and lighter than the TypeScript `repomix` CLI while producing a similarly sized XML pack.
