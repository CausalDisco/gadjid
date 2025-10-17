# Adjustment Identification Distance: A 𝚐𝚊𝚍𝚓𝚒𝚍 for Causal Structure Learning

The 𝚐𝚊𝚍𝚓𝚒𝚍 R wrapper package makes efficient Rust implementations of graph adjustment identification distances (AID) available in R.
These distances (based on ancestor, optimal, and parent adjustment) count how often the respective adjustment identification strategy leads to causal inferences that are incorrect relative to a ground-truth graph when applied to a candidate graph instead.
There is also a [𝚐𝚊𝚍𝚓𝚒𝚍 Python wrapper package](https://pypi.org/project/gadjid/).

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

``` r
install.packages("gadjid")
```

A source package install requires the [rust toolchain to be installed](https://rustup.rs/).


## Get Started Real Quick 🚀 – Introductory Example

Just `install.packages("gadjid")` to install the latest release of 𝚐𝚊𝚍𝚓𝚒𝚍 \

``` r
library(gadjid)

?ancestor_aid

g_true <- rbind(c(0, 1, 1, 1),
                c(0, 0, 1, 1),
                c(0, 0, 0, 1),
                c(0, 0, 0, 0))
g_guess <- rbind(c(0, 1, 0, 0),
                 c(0, 0, 1, 0),
                 c(0, 0, 0, 1),
                 c(0, 0, 0, 0))

ancestor_aid(g_true, g_guess, edge_direction = "from row to column")
oset_aid(g_true, g_guess, edge_direction = "from row to column")
parent_aid(g_true, g_guess, edge_direction = "from row to column")
shd(g_true, g_guess)
sid(g_true, g_guess, edge_direction = "from row to column")
```


### Parallelism – setting the number of threads

𝚐𝚊𝚍𝚓𝚒𝚍 uses [rayon](https://docs.rs/rayon/latest/rayon/) for parallelism
using, per default, as many threads as there are physical CPU cores.
The number of threads to use can be set via the environment variable `RAYON_NUM_THREADS`.
We recommend to do so and to set the number of threads manually,
not least to be explicit and to avoid the small runtime overhead for determining the number of physical CPU cores.


## Implemented Distances

* `ancestor_aid(g_true, g_guess, edge_direction)`
* `oset_aid(g_true, g_guess, edge_direction)`
* `parent_aid(g_true, g_guess, edge_direction)`
* for convenience, the following distances are implemented, too
    * `shd(g_true, g_guess)`
    * `sid(g_true, g_guess, edge_direction)` – only for DAGs!

where `g_true` and `g_guess` are adjacency matrices of a DAG or CPDAG
and `edge_direction` determines whether a `1` at r-th row and c-th column of an adjacency matrix
codes the edge `r → c` (`edge_direction="from row to column"`) or `c → r` (`edge_direction="from column to row"`);
see the documentation pages, such as `?ancestor_aid`, for more information.
The functions are not symmetric in their inputs:
To calculate a distance,
identifying formulas for causal effects are inferred in the graph `g_guess`
and verified against the graph `g_true`.
Distances return a 2-element vector `c(normalised_distance, mistake_count)`
of the fraction of causal effects inferred in g_guess that are wrong relative to g_true, `normalised_distance`,
and the number of wrongly inferred causal effects, `mistake_count`.
There are $p(p-1)$ pairwise causal effects to infer in graphs with $p$ nodes
and we define normalisation as  `normalised_distance = mistake_count / p(p-1)`.

You may also calculate the SID between DAGs via `parent_aid(DAG_true, DAG_guess, edge_direction)`,
but we recommend `ancestor_aid` and `oset_aid` and for CPDAG inputs the `parent_aid` does not coincide with the SID
(see also [our UAI paper](https://doi.org/10.48550/arXiv.2402.08616)).


## LICENSE

𝚐𝚊𝚍𝚓𝚒𝚍 is available in source code form at <https://github.com/CausalDisco/gadjid>.

This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

See also the [MPL-2.0 FAQ](https://www.mozilla.org/MPL/2.0/FAQ).
