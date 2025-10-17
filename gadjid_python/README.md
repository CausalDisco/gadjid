# Adjustment Identification Distance: A 𝚐𝚊𝚍𝚓𝚒𝚍 for Causal Structure Learning

The 𝚐𝚊𝚍𝚓𝚒𝚍 Python wrapper package makes efficient Rust implementations of graph adjustment identification distances (AID) available in Python.
These distances (based on ancestor, optimal, and parent adjustment) count how often the respective adjustment identification strategy leads to causal inferences that are incorrect relative to a ground-truth graph when applied to a candidate graph instead.
There is also a [𝚐𝚊𝚍𝚓𝚒𝚍 R wrapper package](https://cran.r-project.org/package=gadjid).

If you publish research using 𝚐𝚊𝚍𝚓𝚒𝚍, please cite
[our UAI paper](https://doi.org/10.48550/arXiv.2402.08616)
```bibtex
@inproceedings{henckel2024adjustment,
    title = {{Adjustment Identification Distance: A gadjid for Causal Structure Learning}},
    author = {Leonard Henckel and Theo Würtzen and Sebastian Weichwald},
    booktitle = {{Proceedings of the Fortieth Conference on Uncertainty in Artificial Intelligence (UAI)}},
    year = {2024},
    doi = {10.48550/arXiv.2402.08616},
} 
```

> Evaluating graphs learned by causal discovery algorithms is difficult: The number of edges that differ between two graphs does not reflect how the graphs differ with respect to the identifying formulas they suggest for causal effects. We introduce a framework for developing causal distances between graphs which includes the structural intervention distance for directed acyclic graphs as a special case. We use this framework to develop improved adjustment-based distances as well as extensions to completed partially directed acyclic graphs and causal orders. We develop new reachability algorithms to compute the distances efficiently and to prove their low polynomial time complexity. In our package 𝚐𝚊𝚍𝚓𝚒𝚍, we provide implementations of our distances; they are orders of magnitude faster with proven lower time complexity than the structural intervention distance and thereby provide a success metric for causal discovery that scales to graph sizes that were previously prohibitive.


## Installation

Just `pip install gadjid` to install the latest release of 𝚐𝚊𝚍𝚓𝚒𝚍 \
and run `python -c "import gadjid; help(gadjid)"` to get started
(or see [install alternatives](https://github.com/CausalDisco/gadjid#installation--python)).

## Get Started Real Quick 🚀 – Introductory Example

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


### Parallelism – setting the number of threads

𝚐𝚊𝚍𝚓𝚒𝚍 uses [rayon](https://docs.rs/rayon/latest/rayon/) for parallelism
using, per default, as many threads as there are physical CPU cores.
The number of threads to use can be set via the environment variable `RAYON_NUM_THREADS`.
We recommend to do so and to set the number of threads manually,
not least to be explicit and to avoid the small runtime overhead for determining the number of physical CPU cores.


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
The functions are not symmetric in their inputs:
To calculate a distance,
identifying formulas for causal effects are inferred in the graph `Gguess`
and verified against the graph `Gtrue`.
Distances return a tuple `(normalised_distance, mistake_count)`
of the fraction of causal effects inferred in Gguess that are wrong relative to Gtrue, `normalised_distance`,
and the number of wrongly inferred causal effects, `mistake_count`.
There are $p(p-1)$ pairwise causal effects to infer in graphs with $p$ nodes
and we define normalisation as  `normalised_distance = mistake_count / p(p-1)`.

You may also calculate the SID between DAGs via `parent_aid(DAGtrue, DAGguess, edge_direction)`,
but we recommend `ancestor_aid` and `oset_aid` and for CPDAG inputs the `parent_aid` does not coincide with the SID
(see also [our UAI paper](https://doi.org/10.48550/arXiv.2402.08616)).

If `edge_direction="from row to column"`, then
a `1` in row `r` and column `c` codes a directed edge `r → c`;
if `edge_direction="from column to row"`, then
a `1` in row `r` and column `c` codes a directed edge `c → r`;
for either setting of `edge_direction`,
a `2` in row `r` and column `c` codes an undirected edge `r – c`
(an additional `2` in row `c` and column `r` is ignored;
one of the two entries is sufficient to code an undirected edge).

An adjacency matrix for a DAG may only contain 0s and 1s.
An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s.
DAG and CPDAG inputs are validated for acyclicity.
However, for CPDAG inputs, __the user needs to ensure the adjacency
matrix indeed codes a valid CPDAG (instead of just a PDAG)__.


## Empirical Runtime Analysis

Experiments run on a laptop with 8 GB RAM and 4-core i5-8365U processor.
Here, for a graph with $p$ nodes,
sparse graphs have $10p$ edges in expectation,
dense graphs have $0.3p(p-1)/2$ edges in expectation,
and
x-sparse graphs have $0.75p$ edges in expectation.

__Maximum graph size feasible within 1 minute__

| Method       | sparse | dense |
|--------------|-------:|------:|
| Parent-AID   |  13601 |   962 |
| Ancestor-AID |   8211 |   932 |
| Oset-AID     |   1105 |   508 |
| SID in R     |    256 |   239 |

Results obtained with 𝚐𝚊𝚍𝚓𝚒𝚍 v0.1.0 using the Python interface
and the SID R package v1.1 from CRAN.

__Average runtime__
| Method       | x-sparse ($p=1000$) | sparse ($p=256$) | dense ($p=239$) |
|--------------|--------------------:|-----------------:|----------------:|
| Parent-AID   |              7.3 ms |          30.5 ms |          173 ms |
| Ancestor-AID |              3.4 ms |          40.9 ms |          207 ms |
| Oset-AID     |              5.0 ms |           567 ms |         1.68 s  |
| SID in R     |             ~1–2 h  |           ~60 s  |          ~60 s  |

Results obtained with 𝚐𝚊𝚍𝚓𝚒𝚍 v0.1.0 using the Python interface
and the SID R package v1.1 from CRAN.


## LICENSE

𝚐𝚊𝚍𝚓𝚒𝚍 is available in source code form at <https://github.com/CausalDisco/gadjid>.

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

See also the [MPL-2.0 FAQ](https://mozilla.org/MPL/2.0/FAQ).
