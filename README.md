[![test & lint](https://github.com/CausalDisco/gadjid/actions/workflows/test-lint.yml/badge.svg?branch=main)](https://github.com/CausalDisco/gadjid/actions/workflows/test-lint.yml?query=branch%3Amain)
[![PyPi](https://badgen.net/pypi/v/gadjid/?icon=pypi&label=PyPI)](https://pypi.org/project/gadjid)
[![read](https://badgen.net/badge/read/arXiv/b31b1b)](https://doi.org/10.48550/arXiv.2402.08616)


# Adjustment Identification Distance: A 𝚐𝚊𝚍𝚓𝚒𝚍 for Causal Structure Learning

[This is an early release of 𝚐𝚊𝚍𝚓𝚒𝚍 🐥](#this-is-an-early-release-) and feedback is very welcome!

If you publish research using 𝚐𝚊𝚍𝚓𝚒𝚍, please cite
[our article](https://doi.org/10.48550/arXiv.2402.08616)
```bibtex
@article{henckel2024adjustment,
    title = {{Adjustment Identification Distance: A gadjid for Causal Structure Learning}},
    author = {Leonard Henckel and Theo Würtzen and Sebastian Weichwald},
    journal = {{arXiv preprint arXiv:2402.08616}},
    year = {2024},
    doi = {10.48550/arXiv.2402.08616},
}
```


## Get Started Real Quick 🚀

### Installation – Python

Just `pip install gadjid` to install the latest release of 𝚐𝚊𝚍𝚓𝚒𝚍 from [PyPI](https://pypi.org/project/gadjid/) \
and run `python -c "import gadjid; help(gadjid)"` to get started.

#### Install Alternatives

Pip tries to find a matching wheel and install that.
Since we offer precompiled wheels
for most common operating systems, python versions, and CPU architectures,
the installation will usually be quick.
If there is no matching wheel
(or when explicitly installing from source via
`pip install gadjid --no-binary gadjid`),
pip will download the source distribution and compile a wheel for the current platform,
which requires the [rust toolchain to be installed](https://rustup.rs/).

The current development version can be compiled and installed via \
`pip install "git+https://github.com/CausalDisco/gadjid.git"` \
or by cloning this repository and calling either \
`maturin develop --manifest-path ./gadjid_python/Cargo.toml` (unoptimized dev compile)
or \
`maturin develop --manifest-path ./gadjid_python/Cargo.toml --release` (optimized release compile).


### Introductory Example – Python

```python
import gadjid
from gadjid import example, ancestor_aid, oset_aid, parent_aid, shd
import numpy as np

help(gadjid)

example.run_parent_aid()

Gtrue = np.array([
    [0, 1, 1, 1, 1],
    [0, 0, 1, 1, 1],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0]
], dtype=np.int8)
Gguess = np.array([
    [0, 0, 1, 1, 1],
    [1, 0, 1, 1, 1],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0]
], dtype=np.int8)

print(ancestor_aid(Gtrue, Gguess, edge_direction="from row to column"))
print(shd(Gtrue, Gguess))
```


---


𝚐𝚊𝚍𝚓𝚒𝚍 is implemented in Rust
and can conveniently be called from Python via our Python wrapper
(implemented using [maturin](https://www.maturin.rs/) and [PyO3](https://pyo3.rs/)).

> Evaluating graphs learned by causal discovery algorithms is difficult: The number of edges that differ between two graphs does not reflect how the graphs differ with respect to the identifying formulas they suggest for causal effects. We introduce a framework for developing causal distances between graphs which includes the
structural intervention distance for directed acyclic graphs as a special case. We use this framework to develop improved adjustment-based distances as well as extensions to completed partially directed acyclic graphs and causal orders. We develop polynomial-time reachability algorithms to compute the distances efficiently. In our package 𝚐𝚊𝚍𝚓𝚒𝚍, we provide implementations of our distances; they are orders of magnitude faster than the structural intervention distance and thereby provide a success metric for causal discovery that scales to graph sizes that were previously prohibitive.


## This is an Early Release 🐥

* Feedback is very welcome! Just [open an issue](https://github.com/CausalDisco/gadjid/issues/new/choose) on here.
* We are working on making 𝚐𝚊𝚍𝚓𝚒𝚍 available also for R.
* The code is well documented. We plan on making a user and developer documentation available. 📃


## Implemented Distances

* `ancestor_aid(Gtrue, Gguess, edge_direction)`
* `oset_aid(Gtrue, Gguess, edge_direction)`
* `parent_aid(Gtrue, Gguess, edge_direction)`
* for convenience, the following distances are implemented, too
    * `shd(Gtrue, Gguess)`
    * `sid(Gtrue, Gguess, edge_direction)` – only for DAGs!

where `Gtrue` and `Gguess` are adjacency matrices of a DAG or CPDAG
and `edge_direction` determines whether a `1` at r-th row and c-th column of an adjacency matrix
codes the edge `r → c` (`edge_direction="from row to column"`) or `c → r` (`edge_direction="from column to row"`).
The functions are not symmetric in their input:
To calculate a distance,
identifying formulas for causal effects are inferred in the graph `Gguess`
and verified against the graph `Gtrue`.
Distances return a tuple `(normalised_distance, mistake_count)`
of the fraction of causal effects inferred in Gguess that are wrong relative to Gtrue, `normalised_distance`,
and the number of wrongly inferred causal effects, `mistake_count`.
There are $p(p-1)$ pairwise causal effects to infer in graphs with $p$ nodes
and we define normalisation as  `normalised_distance = mistake_count / p(p-1)`.

All graphs are assumed simple, that is, at most one edge is allowed between any two nodes.
An adjacency matrix for a DAG may only contain 0s and 1s;
if `edge_direction="from row to column"`, then
a `1` in row `r` and column `c` codes a directed edge `r → c`;
if `edge_direction="from column to row"`, then
a `1` in row `r` and column `c` codes a directed edge `c → r`;
DAG inputs are validated for acyclicity.
An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s;
for either setting of `edge_direction`,
a `2` in row `r` and column `c` codes an undirected edge `r — c`
(an additional `2` in row `c` and column `r` is ignored; only one of the two entries is required to code an undirected edge);
CPDAG inputs are not validated and __the user needs to ensure the adjacency matrix indeed codes a valid CPDAG (instead of just a PDAG)__.
You may also calculate the SID between DAGs via `parent_aid(DAGtrue, DAGguess, edge_direction)`,
but we recommend `ancestor_aid` and `oset_aid` and for CPDAG inputs the `parent_aid` does not coincide with the SID
(see also our accompanying article).


## Empirical Runtime Analysis

Experiments run on a laptop with 8 GB RAM and 4-core i5-8365U processor.
Here, for a graph with $p$ nodes,
sparse graphs have $10p$ edges in expectation,
dense graphs have $0.3p(p-1)/2$ edges in expectation,
and
sparse graphs have $0.75p$ edges in expectation.

__Maximum graph size feasible within 1 minute__

| Method       | sparse | dense |
|--------------|-------:|------:|
| Parent-AID   |  13005 |   960 |
| Ancestor-AID |   8200 |   932 |
| Oset-AID     |    546 |   250 |
| SID in R     |    255 |   239 |

Results obtained with 𝚐𝚊𝚍𝚓𝚒𝚍 v0.0.1 using the Python interface
and the SID R package v1.1 from CRAN.

__Average runtime__
| Method       | x-sparse ($p=1000$) | sparse ($p=256$) | dense ($p=239$) |
|--------------|--------------------:|-----------------:|----------------:|
| Parent-AID   |              6.3 ms |          22.8 ms |          189 ms |
| Ancestor-AID |              2.7 ms |          38.7 ms |          226 ms |
| Oset-AID     |              3.2 ms |          4.69 s  |         47.3 s  |
| SID in R     |             ~1–2 h  |           ~60 s  |          ~60 s  |

Results obtained with 𝚐𝚊𝚍𝚓𝚒𝚍 v0.0.1 using the Python interface
and the SID R package v1.1 from CRAN.


## Project Structure Overview

* [.github/workflows/](./.github/workflows) – github actions for linting/testing/packaging
* [__gadjid/__](./gadjid/) – Rust core package, which implements
    a graph memory layout purposefully designed for fast memory access in reachability algorithms,
    new reachability algorithms to check the validity of an adjustment set,
    and all DAG/CPDAG distances discussed in the accompanying article
    * [gadjid/src/snapshots](./gadjid/src/snapshots) –
      𝚐𝚊𝚍𝚓𝚒𝚍 is extensively tested (tests at bottom of each `/gadjid/src/**.rs` file)
      and validated against the R implementation of the SID on pairs of DAG inputs (cf. bottom of `parent_aid.rs`);
      this folder holds [insta snapshots](https://insta.rs/) for testing graph and reachability algorithms and all distances against
      (cf. [gadjid/src/lib.rs](./gadjid/src/lib.rs))
* [gadjid_python/](./gadjid_python/) –
    python wrapper that accepts numpy and scipy int8 matrices as graph adjacency matrices
    * [gadjid_python/tests/](./gadjid_python/tests/) – runs tests of and via the python 𝚐𝚊𝚍𝚓𝚒𝚍 wrapper:
        1. tests the loading of numpy arrays and views as well as scipy sparse csr/csc matrices
        2. tests `parent_aid` against the R implementation of the SID on pairs of DAG inputs;
        since in the special case of DAG inputs the Parent-AID coincides with the SID,
        this end-to-end tests the check for validity of adjustment sets implemented via new reachability algorithms
* [gadjid_r/](./gadjid_r/) – placeholder for the R wrapper to come!
* [testgraphs/](./testgraphs/) – testgraphs in .mtx files (Matrix Market Exchange Format), csv files with the SHD/SID between the testgraphs to test against, checksums


## LICENSE

𝚐𝚊𝚍𝚓𝚒𝚍 is available in source code form at <https://github.com/CausalDisco/gadjid>.

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

See also the [MPL-2.0 FAQ](https://mozilla.org/MPL/2.0/FAQ).
