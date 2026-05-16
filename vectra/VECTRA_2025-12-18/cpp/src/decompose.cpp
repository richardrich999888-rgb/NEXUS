/**
 * @file decompose.cpp
 * @brief Payload decomposition: D → (S, V)
 */

#include "vectra/vectra.hpp"

namespace vectra::detail {

Result<DecompositionResult> decompose(const Payload& payload) {
    DecompositionResult result;

    if (payload.empty()) {
        return result;
    }

    // Simple implementation: treat all as variable
    // Full implementation would find structural patterns
    VariableSegment segment;
    segment.range = ByteRange{0, payload.size()};
    segment.data = payload.data();
    segment.semantic_type = SemanticType::Opaque;

    result.variable.segments.push_back(std::move(segment));
    result.structure.levels.push_back(StructureLevel{0, {}, {}});

    return result;
}

Result<Payload> recompose(const Structure& structure, const VariablePart& variable) {
    size_t struct_max = 0;
    for (const auto& r : structure.byte_ranges) {
        struct_max = std::max(struct_max, r.end);
    }

    size_t var_max = 0;
    for (const auto& s : variable.segments) {
        var_max = std::max(var_max, s.range.end);
    }

    const size_t total_len = std::max(struct_max, var_max);
    if (total_len == 0) {
        return Payload{};
    }

    std::vector<uint8_t> output(total_len, 0);

    // Place structural components
    for (size_t i = 0; i < structure.byte_ranges.size() && i < structure.levels.size(); ++i) {
        const auto& range = structure.byte_ranges[i];
        const auto& level = structure.levels[i];
        const size_t len = range.length();
        if (level.literals.size() >= len) {
            std::copy_n(level.literals.begin(), len, output.begin() + range.start);
        }
    }

    // Place variable components
    for (const auto& segment : variable.segments) {
        const size_t len = segment.range.length();
        if (segment.data.size() >= len) {
            std::copy_n(segment.data.begin(), len, output.begin() + segment.range.start);
        }
    }

    return Payload{std::move(output)};
}

}  // namespace vectra::detail
