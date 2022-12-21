@echo off
cls
echo generating
type tracing.folded | inferno-flamegraph > tracing-flamegraph.svg
echo done