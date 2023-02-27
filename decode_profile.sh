#!/bin/bash

perf record -g --call-graph dwarf ./target/debug/huffman ./bird.o -d -o bird.txt


perf script > perf.export
