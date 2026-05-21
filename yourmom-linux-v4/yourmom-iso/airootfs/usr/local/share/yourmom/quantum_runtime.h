// quantum_runtime.h - Quantum superposition support for .yourmom
// "Yo mama so fat, she exists in multiple states simultaneously"

#ifndef QUANTUM_RUNTIME_H
#define QUANTUM_RUNTIME_H

#include <stdlib.h>
#include <stdio.h>
#include <time.h>
#include <signal.h>
#include <string.h>

// Quantum integer - can exist in superposition
typedef struct {
    int* values;       // Possible values
    int count;         // Number of possibilities
    int collapsed_idx; // -1 if still in superposition, else index of collapsed value
    int collapsed_val; // The actual collapsed value (cached)
} quantum_int;

// Global RNG state
static int _quantum_rng_initialized = 0;

void _quantum_init_rng() {
    if (!_quantum_rng_initialized) {
        srand(time(NULL));
        _quantum_rng_initialized = 1;
    }
}

// Create quantum int from single value (classical)
static inline quantum_int _quantum_create(int val) {
    quantum_int q;
    q.values = malloc(sizeof(int));
    q.values[0] = val;
    q.count = 1;
    q.collapsed_idx = 0;
    q.collapsed_val = val;
    return q;
}

// Comparison helpers that handle temporaries
static inline int _quantum_eq_val(quantum_int* a, int b_val) {
    quantum_int b = _quantum_create(b_val);
    int result = _quantum_eq(a, &b);
    free(b.values);
    return result;
}

static inline int _quantum_lt_val(quantum_int* a, int b_val) {
    quantum_int b = _quantum_create(b_val);
    int result = _quantum_lt(a, &b);
    free(b.values);
    return result;
}

static inline int _quantum_gt_val(quantum_int* a, int b_val) {
    quantum_int b = _quantum_create(b_val);
    int result = _quantum_gt(a, &b);
    free(b.values);
    return result;
}

// Create quantum int from multiple possibilities
quantum_int _quantum_superposition(int* vals, int count) {
    _quantum_init_rng();
    quantum_int q;
    q.values = malloc(sizeof(int) * count);
    memcpy(q.values, vals, sizeof(int) * count);
    q.count = count;
    q.collapsed_idx = -1;  // In superposition
    q.collapsed_val = 0;
    return q;
}

// Observe/collapse a quantum int
int _quantum_observe(quantum_int* q) {
    if (q->collapsed_idx == -1) {
        // Collapse the wavefunction
        _quantum_init_rng();
        q->collapsed_idx = rand() % q->count;
        q->collapsed_val = q->values[q->collapsed_idx];
    }
    return q->collapsed_val;
}

// Check if already collapsed
int _quantum_is_collapsed(quantum_int* q) {
    return q->collapsed_idx != -1;
}

// Get value (forces observation)
int _quantum_get(quantum_int* q) {
    return _quantum_observe(q);
}

// Quantum arithmetic - operates on collapsed values
quantum_int _quantum_add(quantum_int* a, quantum_int* b) {
    return _quantum_create(_quantum_get(a) + _quantum_get(b));
}

quantum_int _quantum_sub(quantum_int* a, quantum_int* b) {
    return _quantum_create(_quantum_get(a) - _quantum_get(b));
}

quantum_int _quantum_mul(quantum_int* a, quantum_int* b) {
    return _quantum_create(_quantum_get(a) * _quantum_get(b));
}

quantum_int _quantum_div(quantum_int* a, quantum_int* b) {
    return _quantum_create(_quantum_get(a) / _quantum_get(b));
}

// Quantum comparisons (force observation)
int _quantum_eq(quantum_int* a, quantum_int* b) {
    return _quantum_get(a) == _quantum_get(b);
}

int _quantum_lt(quantum_int* a, quantum_int* b) {
    return _quantum_get(a) < _quantum_get(b);
}

int _quantum_gt(quantum_int* a, quantum_int* b) {
    return _quantum_get(a) > _quantum_get(b);
}

// Error handler - prints yo mama jokes on crash
static char* _error_jokes[] = {
    "Yo mama so fat, she caused a stack overflow",
    "Yo mama so dumb, she dereferenced a NULL pointer", 
    "Yo mama so ugly, even the CPU refused to execute",
    "Yo mama so slow, she caused a heap corruption",
    "Yo mama so cheap, she freed memory twice",
    "Yo mama so fat, she exceeded the address space",
    "Yo mama so random, she corrupted the RNG seed"
};

void _yo_mama_so_broken(int sig) {
    _quantum_init_rng();
    fprintf(stderr, "\n💀 SEGFAULT: %s\n", _error_jokes[rand() % 7]);
    exit(1);
}

// Initialize error handlers
void _quantum_init_errors() {
    signal(SIGSEGV, _yo_mama_so_broken);
    signal(SIGABRT, _yo_mama_so_broken);
    signal(SIGFPE, _yo_mama_so_broken);
}

// Helper for creating superposition from literals
#define SUPERPOSITION(...) ({ \
    int _vals[] = {__VA_ARGS__}; \
    _quantum_superposition(_vals, sizeof(_vals)/sizeof(int)); \
})

// Helper to wrap int as quantum_int
static inline quantum_int _qint(int val) {
    return _quantum_create(val);
}

#endif // QUANTUM_RUNTIME_H
