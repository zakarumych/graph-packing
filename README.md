# graph-packing

`pack_partial_order` packs sized objects in 1D with a partial order relation:

- comparable pairs may overlap;
- incomparable pairs are strictly separated.

Complexity:

- time: exponential in the number of objects in the worst case (exact optimal search), with `O(n^2)` preprocessing;
- space: `O(n^2)` for the incomparability matrix and `O(n)` recursion state.
