// Access-only bridge overlaid into the pinned Go standard-library package.
package slices

func S06Heap[E any](data []E, start, end int, compare func(E, E) int) {
	pdqsortCmpFunc(data, start, end, 0, compare)
}
func S06Partial[E any](data []E, start, end int, compare func(E, E) int) bool {
	return partialInsertionSortCmpFunc(data, start, end, compare)
}
func S06Break[E any](data []E, start, end int, compare func(E, E) int) {
	breakPatternsCmpFunc(data, start, end, compare)
}
func S06PartitionEqual[E any](data []E, start, end, pivot int, compare func(E, E) int) int {
	return partitionEqualCmpFunc(data, start, end, pivot, compare)
}
