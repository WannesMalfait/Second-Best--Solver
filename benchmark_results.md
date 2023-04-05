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
