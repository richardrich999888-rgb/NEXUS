/**
 * @file artifact.cpp
 * @brief Artifact serialization implementation.
 */

#include "vectra/vectra.hpp"

#include <cstring>

namespace vectra {

// Simple binary serialization (production would use proper format)
std::vector<uint8_t> Artifact::to_bytes() const {
    std::vector<uint8_t> result;

    // Version marker
    uint64_t version = VERSION_ID;
    for (int i = 0; i < 8; ++i) {
        result.push_back(static_cast<uint8_t>(version >> (i * 8)));
    }

    // Generator base length and data
    uint32_t base_len = static_cast<uint32_t>(generator.base.size());
    for (int i = 0; i < 4; ++i) {
        result.push_back(static_cast<uint8_t>(base_len >> (i * 8)));
    }
    result.insert(result.end(), generator.base.begin(), generator.base.end());

    // Repetition spec
    for (int i = 0; i < 4; ++i) {
        result.push_back(static_cast<uint8_t>(generator.repetition.count >> (i * 8)));
    }
    for (int i = 0; i < 4; ++i) {
        result.push_back(static_cast<uint8_t>(generator.repetition.stride >> (i * 8)));
    }

    // Residual segments count
    uint32_t seg_count = static_cast<uint32_t>(residual.segments.size());
    for (int i = 0; i < 4; ++i) {
        result.push_back(static_cast<uint8_t>(seg_count >> (i * 8)));
    }

    // Each segment
    for (const auto& seg : residual.segments) {
        // Range
        uint64_t start = seg.range.start;
        uint64_t end = seg.range.end;
        for (int i = 0; i < 8; ++i) {
            result.push_back(static_cast<uint8_t>(start >> (i * 8)));
        }
        for (int i = 0; i < 8; ++i) {
            result.push_back(static_cast<uint8_t>(end >> (i * 8)));
        }

        // Delta length and data
        uint32_t delta_len = static_cast<uint32_t>(seg.delta.size());
        for (int i = 0; i < 4; ++i) {
            result.push_back(static_cast<uint8_t>(delta_len >> (i * 8)));
        }
        result.insert(result.end(), seg.delta.begin(), seg.delta.end());
    }

    // Constraints
    uint64_t output_len = constraints.output_length;
    for (int i = 0; i < 8; ++i) {
        result.push_back(static_cast<uint8_t>(output_len >> (i * 8)));
    }
    result.insert(result.end(), constraints.output_hash.begin(), constraints.output_hash.end());

    // Integrity
    result.insert(result.end(), integrity.payload_hash.begin(), integrity.payload_hash.end());
    result.insert(result.end(), integrity.artifact_hash.begin(), integrity.artifact_hash.end());
    for (int i = 0; i < 8; ++i) {
        result.push_back(static_cast<uint8_t>(integrity.version >> (i * 8)));
    }
    for (int i = 0; i < 8; ++i) {
        result.push_back(static_cast<uint8_t>(integrity.encoded_at >> (i * 8)));
    }

    return result;
}

Artifact Artifact::from_bytes(const std::vector<uint8_t>& data) {
    Artifact artifact;
    size_t pos = 0;

    auto read_u32 = [&]() -> uint32_t {
        uint32_t val = 0;
        for (int i = 0; i < 4 && pos < data.size(); ++i, ++pos) {
            val |= static_cast<uint32_t>(data[pos]) << (i * 8);
        }
        return val;
    };

    auto read_u64 = [&]() -> uint64_t {
        uint64_t val = 0;
        for (int i = 0; i < 8 && pos < data.size(); ++i, ++pos) {
            val |= static_cast<uint64_t>(data[pos]) << (i * 8);
        }
        return val;
    };

    auto read_bytes = [&](size_t len) -> std::vector<uint8_t> {
        std::vector<uint8_t> result;
        for (size_t i = 0; i < len && pos < data.size(); ++i, ++pos) {
            result.push_back(data[pos]);
        }
        return result;
    };

    auto read_hash = [&]() -> Hash256 {
        Hash256 hash{};
        for (size_t i = 0; i < 32 && pos < data.size(); ++i, ++pos) {
            hash[i] = data[pos];
        }
        return hash;
    };

    // Version
    read_u64();  // Skip version marker

    // Generator
    uint32_t base_len = read_u32();
    artifact.generator.base = read_bytes(base_len);
    artifact.generator.repetition.count = read_u32();
    artifact.generator.repetition.stride = read_u32();

    // Residual segments
    uint32_t seg_count = read_u32();
    for (uint32_t i = 0; i < seg_count; ++i) {
        ResidualSegment seg;
        seg.range.start = read_u64();
        seg.range.end = read_u64();
        uint32_t delta_len = read_u32();
        seg.delta = read_bytes(delta_len);
        artifact.residual.segments.push_back(std::move(seg));
    }

    // Constraints
    artifact.constraints.output_length = read_u64();
    artifact.constraints.output_hash = read_hash();

    // Integrity
    artifact.integrity.payload_hash = read_hash();
    artifact.integrity.artifact_hash = read_hash();
    artifact.integrity.version = read_u64();
    artifact.integrity.encoded_at = read_u64();

    // Predictor state (default)
    artifact.predictor_state.version = VERSION_ID;

    return artifact;
}

}  // namespace vectra
