# Benchmark Results

This file keeps track of the perfomance of the solver by storing the results of the benchmarks at various versions of the solver. The result is the output after running `bench 6`

## Simple Alpha-Beta pruning:

```terminal
Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 2..5

Finished benchmark:
Average time: 0.0012s
Average number of nodes searched: 5145.37
Average knps: 4257.93 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 4..7

Finished benchmark:
Average time: 0.0203s
Average number of nodes searched: 93832.80
Average knps: 4627.06 knps
``` 

```terminal

solver speed (depth 7)  time:   [34.095 ms 34.390 ms 34.756 ms]
solver speed (depth 9)  time:   [272.22 ms 273.93 ms 275.98 ms]
solver speed end (depth 7)
                        time:   [77.575 ms 77.664 ms 77.763 ms]
solver speed end (depth 9)
                        time:   [2.3153 s 2.3237 s 2.3336 s]
solver efficiency (depth 7)
                        time:   [23.680 ms 24.149 ms 24.663 ms]
solver efficiency (depth 9)
                        time:   [217.50 ms 217.66 ms 217.90 ms]
```

## PV-search with Iterative Deepening

This actually makes the solver slower, but paves the way for future optimizations.

```terminal
Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 2..5

Finished benchmark:
Average time: 0.0016s
Average number of nodes searched: 5248.44
Average knps: 3323.36 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 4..7

Finished benchmark:
Average time: 0.0322s
Average number of nodes searched: 126056.65
Average knps: 3917.16 knps 
```

```terminal

solver speed (depth 7)  time:   [45.786 ms 47.076 ms 48.473 ms]
solver speed (depth 9)  time:   [482.47 ms 497.01 ms 512.42 ms]
solver speed end (depth 7)
                        time:   [283.97 ms 289.99 ms 296.54 ms]
solver speed end (depth 9)
                        time:   [4.9764 s 5.0524 s 5.1339 s]
solver efficiency (depth 7)
                        time:   [26.902 ms 27.697 ms 28.552 ms]
solver efficiency (depth 9)
                        time:   [413.07 ms 418.46 ms 424.54 ms]
```

## Bitboard representation

Although this shouldn't have impacted node counts, because of some bugs being fixed and the benchmarks being regenerated, the node counts are different. However, the speed benchmarks clearly show an increase in speed for a fixed depth and position.

```terminal

Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 2..5

Finished benchmark:
Average time: 0.0010s
Average number of nodes searched: 1953.44
Average knps: 1897.08 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 4..7

Finished benchmark:
Average time: 0.0054s
Average number of nodes searched: 51074.60
Average knps: 9372.14 knps
```


```terminal
 
solver speed (depth 7)  time:   [17.139 ms 17.299 ms 17.472 ms]
                        change: [-64.366% -63.253% -62.164%] (p = 0.00 < 0.05)
                        Performance has improved.
solver speed (depth 9)  time:   [188.25 ms 191.84 ms 195.89 ms]
                        change: [-62.769% -61.402% -60.022%] (p = 0.00 < 0.05)
                        Performance has improved.
solver speed end (depth 7)
                        time:   [63.427 ms 64.273 ms 65.255 ms]
                        change: [-78.410% -77.836% -77.263%] (p = 0.00 < 0.05)
                        Performance has improved.
solver speed end (depth 9)
                        time:   [1.1833 s 1.1958 s 1.2090 s]
                        change: [-76.763% -76.333% -75.880%] (p = 0.00 < 0.05)
                        Performance has improved.
solver efficiency (depth 7)
                        time:   [11.991 ms 12.230 ms 12.501 ms]
                        change: [-60.349% -58.617% -56.794%] (p = 0.00 < 0.05)
                        Performance has improved.
solver efficiency (depth 9)
                        time:   [170.94 ms 174.59 ms 178.86 ms]
                        change: [-59.365% -58.278% -56.960%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Move sorting

Again, the bench marks had to be regenerated, since there were some bugs in the previous version.

This adds some heuristics for which moves should be looked at first. As you can see, the number of explored nodes goes down dramatically, due to better pruning.

```terminal
Starting benchmark with 1000 positions.
number of moves: 16..40
solution depth: 6..10

Finished benchmark:
Average time: 0.0231s
Average number of nodes searched: 188024.96
Average knps: 8132.64 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 2..5

Finished benchmark:
Average time: 0.0003s
Average number of nodes searched: 272.00
Average knps: 801.24 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 4..7

Finished benchmark:
Average time: 0.0012s
Average number of nodes searched: 5334.01
Average knps: 4351.29 knps```

```terminal

solver speed (depth 7)  time:   [18.713 ms 18.731 ms 18.754 ms]
solver speed (depth 9)  time:   [69.188 ms 69.621 ms 70.197 ms]
solver speed end (depth 7)
                        time:   [8.1593 ms 8.1743 ms 8.1922 ms]
solver speed end (depth 9)
                        time:   [91.788 ms 91.869 ms 91.987 ms]
solver efficiency (depth 7)
                        time:   [923.92 µs 924.50 µs 925.15 µs]
solver efficiency (depth 9)
                        time:   [5.0140 ms 5.0215 ms 5.0313 ms]
```

## Transposition Table

By caching explored nodes, we prevent doing the same work more than once. The number of explored nodes goes down significantly. As a consequence, the overall speed of the solver increases, even though it is spending more time on each node.
```terminal

Starting benchmark with 1000 positions.
number of moves: 16..40
solution depth: 6..10

Finished benchmark:
Average time: 0.0049s
Average number of nodes searched: 26108.30
Average knps: 5308.18 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 2..5

Finished benchmark:
Average time: 0.0000s
Average number of nodes searched: 216.63
Average knps: 5148.78 knps


Starting benchmark with 1000 positions.
number of moves: 0..40
solution depth: 4..7

Finished benchmark:
Average time: 0.0004s
Average number of nodes searched: 2433.18
Average knps: 6358.06 knps
```

```terminal

solver speed (depth 7)  time:   [11.983 ms 12.379 ms 12.818 ms]
                        change: [-29.708% -27.468% -25.096%] (p = 0.00 < 0.05)
                        Performance has improved.
solver speed (depth 9)  time:   [61.901 ms 63.369 ms 64.948 ms]
                        change: [-2.5731% -0.3737% +2.0823%] (p = 0.77 > 0.05)
                        No change in performance detected.
solver speed end (depth 7)
                        time:   [2.0684 ms 2.1453 ms 2.2360 ms]
                        change: [-72.900% -72.211% -71.550%] (p = 0.00 < 0.05)
                        Performance has improved.
solver speed end (depth 9)
                        time:   [13.870 ms 14.211 ms 14.579 ms]
                        change: [-83.992% -83.601% -83.156%] (p = 0.00 < 0.05)
                        Performance has improved.
solver efficiency (depth 7)
                        time:   [838.88 µs 854.19 µs 870.53 µs]
                        change: [-4.0314% -2.2746% -0.5558%] (p = 0.02 < 0.05)
                        Change within noise threshold.
solver efficiency (depth 9)
                        time:   [3.3862 ms 3.5350 ms 3.6835 ms]
                        change: [-29.358% -27.534% -25.490%] (p = 0.00 < 0.05)
                        Performance has improved.
```