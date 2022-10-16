# imara-diff

[![crates.io](https://img.shields.io/crates/v/imara-diff?style=flat-square)](https://crates.io/crates/imara-diff)
![crates.io](https://img.shields.io/crates/l/imara-diff?style=flat-square)

Imara-diff is a solid (imara in swahili) diff library for rust.
Solid refers to the fact that imara-diff provides very good runtime performance even
in pathologic cases so that your application never appears to freeze while waiting on a diff.
The performance improvements are achieved using battle tested heuristics used in gnu-diff and git
that are known to yield fast runtime and performance.

Imara-diff is also designed to be flexible so that it can be used with arbitrary collections and
not just lists and strings and even allows reusing large parts of the computation when
comparing the same file to multiple different files.

Imara-diff provides two diff algorithms:

* The linear-space variant of the well known [**myer** algorithm](http://www.xmailserver.org/diff2.pdf) 
* The **histogram** algorithm which variant of the patience diff algorithm.

Myers algorithm has been enhanced with preprocessing and multiple heuristics to ensure fast runtime in pathological 
cases to avoid quadratic time complexity and closely matches the behaviour of gnu-diff and git.
The histogram algorithm was originally ported from git but has been heavily optimized.
The **histogram algorithm outperforms myers diff** by 10% - 100% across a **wide variety of workloads**.

## Benchmarks

The most used diffing libraries in the rust ecosystem are [similar](https://crates.io/crates/similar) and [dissimilar](https://crates.io/crates/similar).
The fastest diff implementation both of these offer is a simple implementation of the myer algorithm
without preprocessing or additional heuristics.
As these implementations are very similar only `similar` was included in the benchmark.

To provide a benchmark to reflects real-world workloads, the git history of different open source projects were used.
For each repo two (fairly different) tags were chosen.
A tree diff is performed with [gitoxide](https://github.com/Byron/gitoxide) and the pairs of files that should be saved are stored in memory.
The diffs collected using this method are often fairly large, because the repositories are compared over a large span of time.
Therefore, the tree diff of the last 30 commit before the tag (equivalent of `git diff TAG^ TAG`, `git diff TAG^^ TAG^^`) were also used to also include smaller diffs.

The benchmark measures the runtime of performing a **line diff** between the collected files.
As a measure of complexity for each change `(M + N) D` was used where `M` and `N` are the lengths of the two compared files
and `D` is the length of the edit script required to transform these files into each other (determined with myers algorithm).
This complexity measure is used to divide the changes into 10 badges.
The time to compute the line diffs in each badge was benchmarked.

The plots below show the runtime for each **average** complexity (runtime is normalized by the number of diffs).
Note that these plots are shown in logarithmic scale due to the large runtime of `similar` for complex diffs.
Furthermore, to better highlight the performance of the histogram algorithm, the speedup of the histogram algorithm
compared to the myers algorithm is shown separately.

* [Linux](#Linux)
* [Rust](#Rust)
* [VSCode](#VSCode)
* [Helix](#Helix)

# Linux

The sourcecode of the linux kernel.

- **Repo** - https://kernel.org
- **Tags** - `v5.7` and `v6.0`

<img src='plots/linux_comparison.svg' width="700">
<img src='plots/linux_speedup.svg' width="700">

# Rust

The sourcecode of the rust compiler, standard library and various related tooling.

- **Repo** - https://github.com/rust-lang/rust
- **Tags** - `1.50.0` and `1.64.0`

<img src='plots/rust_comparison.svg' width="700">
<img src='plots/rust_speedup.svg' width="700">

# VScode

The sourcecode of the vscode editor.

- **Repo** - https://github.com/microsoft/vscode
- **Tags** - `1.41.0` and `1.72.2`

<img src='plots/vscode_comparison.svg' width="700">
<img src='plots/vscode_speedup.svg' width="700">

# Helix

The sourcecode of the helix editor.

- **Repo** - https://github.com/helix-editor/helix
- **Tags** - `v0.5.0` and `22.08.1`

<img src='plots/helix_comparison.svg' width="700">
<img src='plots/helix_speedup.svg' width="700">
