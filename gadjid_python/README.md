# Adjustment Identification Distance: A 𝚐𝚊𝚍𝚓𝚒𝚍 for Causal Structure Learning

This is an early release candidate 𝚐𝚊𝚍𝚓𝚒𝚍 0.0.1-rc.0 🐥 and feedback is very welcome!


## Get Started Real Quick 🚀 – Introductory Example

```python
from gadjid import example, parent_aid, ancestor_aid, oset_aid, shd
import numpy as np

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

print(ancestor_aid(Gtrue, Gguess))
print(shd(Gtrue, Gguess))
```


---


𝚐𝚊𝚍𝚓𝚒𝚍 is implemented in Rust
and can conveniently be called from Python via our Python wrapper
(implemented using [maturin](https://www.maturin.rs/) and [PyO3](https://pyo3.rs/)).

> Evaluating graphs learned by causal discovery algorithms is difficult: The number of edges that differ between two graphs does not reflect how the graphs differ with respect to the identifying formulas they suggest for causal effects. We introduce a framework for developing causal distances between graphs which includes the
structural intervention distance for directed acyclic graphs as a special case. We use this framework to develop improved adjustment-based distances as well as extensions to completed partially directed acyclic graphs and causal orders. We develop polynomial-time reachability algorithms to compute the distances efficiently. In our package 𝚐𝚊𝚍𝚓𝚒𝚍, we provide implementations of our distances; they are orders of magnitude faster than the structural intervention distance and thereby provide a success metric for causal discovery that scales to graph sizes that were previously prohibitive.


## Implemented Distances

* `parent_aid(Gtrue, Gguess)`
* `ancestor_aid(Gtrue, Gguess)`
* `oset_aid(Gtrue, Gguess)`
* for convenience, the following distances are implemented, too
    * `shd(Gtrue, Gguess)`
    * `sid(Gtrue, Gguess)` – only for DAGs!

where Gtrue and Gguess are adjacency matrices of a DAG or CPDAG.
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
a `1` in row `s` and column `t` codes a directed edge `Xₛ → Xₜ`;
DAG inputs are validated for acyclicity.
An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s;
a `2` in row `s` and column `t` codes a undirected edge `Xₛ — Xₜ`
(an additional `2` in row `t` and column `s` is ignored; only one of the two entries is required to code an undirected edge);
CPDAG inputs are not validated and __the user needs to ensure the adjacency matrix indeed codes a valid CPDAG (instead of just a PDAG)__.
You may also calculate the SID between DAGs via `parent_aid(DAGtrue, DAGguess)`,
but we recommend `ancestor_aid` and `oset_aid` and for CPDAG inputs our `parent_aid` does not coincide with the SID
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

__Average runtime__
| Method       | x-sparse ($p=1000$) | sparse ($p=256$) | dense ($p=239$) |
|--------------|--------------------:|-----------------:|----------------:|
| Parent-AID   |              6.3 ms |          22.8 ms |          189 ms |
| Ancestor-AID |              2.7 ms |          38.7 ms |          226 ms |
| Oset-AID     |              3.2 ms |          4.69 s  |         47.3 s  |
| SID in R     |             ~1–2 h  |           ~60 s  |          ~60 s  |


## LICENSE

gadjid is available in source code form at <https://github.com/CausalDisco/gadjid>.

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

See also the [MPL-2.0 FAQ](https://mozilla.org/MPL/2.0/FAQ).
