# Standard build for testing frameworks such as OpenBench:
#   make [EXE=<output name>] [EVALFILE=<network file>]
# Without EVALFILE the network in nets/default.nnue is embedded.

EXE ?= trinity
ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

ifdef EVALFILE
	export EVALFILE
endif

.PHONY: all
all:
	cargo rustc --release --bin trinity -- -C target-cpu=native --emit link=$(NAME)
