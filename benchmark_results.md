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
