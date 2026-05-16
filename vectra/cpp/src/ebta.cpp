/**
 * @file ebta.cpp
 * @brief EBTA — Entropy-Bounded Tensor Algebra implementation.
 *
 * Implements spec §6: Entropy constraint enforcement.
 */

#include "vectra/vectra.hpp"

#include <cmath>
#include <numeric>

namespace vectra {

double compute_byte_entropy(const std::vector<uint8_t>& data) {
    if (data.empty()) {
        return 0.0;
    }

    // Count byte frequencies
    std::array<uint64_t, 256> counts{};
    for (uint8_t b : data) {
        counts[b]++;
    }

    const double total = static_cast<double>(data.size());
    double entropy = 0.0;

    for (uint64_t count : counts) {
        if (count > 0) {
            const double p = static_cast<double>(count) / total;
            entropy -= p * std::log2(p);
        }
    }

    return entropy;
}

namespace detail {

EbtaResult ebta_validate(const Residual& residual, double h_max) {
    // Collect all residual bytes
    std::vector<uint8_t> all_bytes;
    for (const auto& segment : residual.segments) {
        all_bytes.insert(all_bytes.end(), segment.delta.begin(), segment.delta.end());
    }

    const double entropy = compute_byte_entropy(all_bytes);

    return EbtaResult{
        .valid = entropy <= h_max,
        .entropy = entropy,
        .max_entropy = h_max
    };
}

}  // namespace detail

}  // namespace vectra
