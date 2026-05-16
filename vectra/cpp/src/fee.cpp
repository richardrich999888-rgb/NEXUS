/**
 * @file fee.cpp
 * @brief FEE — Fractal Entropy Encoding implementation.
 */

#include "vectra/vectra.hpp"

namespace vectra::detail {

Result<FeeResult> fee_encode(const Structure& structure) {
    FeeResult result;

    if (structure.levels.empty()) {
        result.generator.repetition = RepetitionSpec{0, 0};
        return result;
    }

    const auto& base_level = structure.levels[0];
    result.generator.base = base_level.literals;

    // Calculate repetition
    if (structure.byte_ranges.empty()) {
        result.generator.repetition = RepetitionSpec{0, 0};
    } else if (structure.byte_ranges.size() == 1) {
        result.generator.repetition = RepetitionSpec{
            1, static_cast<uint32_t>(base_level.literals.size())
        };
    } else {
        const uint32_t stride = static_cast<uint32_t>(
            structure.byte_ranges[1].start - structure.byte_ranges[0].start
        );
        result.generator.repetition = RepetitionSpec{
            static_cast<uint32_t>(structure.byte_ranges.size()), stride
        };
    }

    return result;
}

Structure regenerate_structure(const Generator& generator, const MappingSet& mappings) {
    Structure result;

    // Generate base level
    StructureLevel base_level;
    base_level.pattern_id = 0;
    base_level.literals = generator.base;
    result.levels.push_back(std::move(base_level));

    // Generate byte ranges from repetition spec
    const size_t pattern_len = generator.base.size();
    for (uint32_t i = 0; i < generator.repetition.count; ++i) {
        const size_t start = i * generator.repetition.stride;
        result.byte_ranges.push_back(ByteRange{start, start + pattern_len});
    }

    // Apply mappings (simplified)
    for (const auto& mapping : mappings.mappings) {
        if (mapping.from_level < result.levels.size()) {
            const auto& source = result.levels[mapping.from_level];
            StructureLevel derived;
            derived.pattern_id = source.pattern_id + 1;

            switch (mapping.transform) {
                case MappingTransform::Identity:
                    derived.literals = source.literals;
                    break;
                case MappingTransform::Offset: {
                    const int32_t offset = std::get<int32_t>(mapping.transform_param);
                    derived.literals.reserve(source.literals.size());
                    for (uint8_t b : source.literals) {
                        derived.literals.push_back(static_cast<uint8_t>(b + offset));
                    }
                    break;
                }
                case MappingTransform::Concat: {
                    const auto& indices = std::get<std::vector<size_t>>(mapping.transform_param);
                    for (size_t idx : indices) {
                        (void)idx;  // indices not used in this simplified version
                        derived.literals.insert(derived.literals.end(),
                                               source.literals.begin(),
                                               source.literals.end());
                    }
                    break;
                }
            }

            result.levels.push_back(std::move(derived));
        }
    }

    return result;
}

}  // namespace vectra::detail
