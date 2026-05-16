/**
 * @file encode.cpp
 * @brief Top-level encoding implementation.
 *
 * Implements spec §1: E : 𝒟 → 𝒜 ∪ 𝒟
 */

#include "vectra/vectra.hpp"

#include <openssl/sha.h>

namespace vectra {

// =============================================================================
// SHA-256 Implementation
// =============================================================================

Hash256 sha256(const std::vector<uint8_t>& data) {
    Hash256 hash{};
    SHA256(data.data(), data.size(), hash.data());
    return hash;
}

// =============================================================================
// Utility Functions
// =============================================================================

bool can_decode(const Artifact& artifact) noexcept {
    return artifact.integrity.version == VERSION_ID;
}

size_t estimate_artifact_size(const Artifact& artifact) {
    size_t size = 0;

    // Generator
    size += artifact.generator.base.size();
    size += 8;  // repetition spec

    // Mappings (estimate)
    size += artifact.mappings.mappings.size() * 24;

    // Predictor state (estimate)
    size += 64;

    // Residual
    for (const auto& segment : artifact.residual.segments) {
        size += segment.delta.size();
        size += 16;  // range metadata
    }

    // Constraints
    size += 8 + 32;  // length + hash

    // Integrity
    size += 32 + 32 + 8 + 8;  // hashes + version + timestamp

    return size;
}

double compression_ratio(size_t original_size, const Artifact& artifact) {
    const size_t artifact_size = estimate_artifact_size(artifact);
    if (artifact_size == 0) {
        return 1.0;
    }
    return static_cast<double>(original_size) / static_cast<double>(artifact_size);
}

// =============================================================================
// Encoding Implementation
// =============================================================================

EncodeResult vectra_encode(Payload payload) {
    // Step 1: Decompose
    auto decompose_result = detail::decompose(payload);
    if (!decompose_result) {
        return EncodeResult::pass_through(std::move(payload));
    }
    auto& [structure, variable] = *decompose_result;

    // Step 2: FEE encode
    auto fee_result = detail::fee_encode(structure);
    if (!fee_result) {
        return EncodeResult::pass_through(std::move(payload));
    }
    auto& [generator, mappings] = *fee_result;

    // Step 3: NSGE predict
    auto nsge_result = detail::nsge_predict(variable);
    if (!nsge_result) {
        return EncodeResult::pass_through(std::move(payload));
    }
    auto& [predicted, predictor_state] = *nsge_result;

    // Step 4: Compute residual
    Residual residual;
    for (size_t i = 0; i < variable.segments.size() && i < predicted.segments.size(); ++i) {
        const auto& actual = variable.segments[i];
        const auto& pred = predicted.segments[i];

        if (actual.data.size() != pred.data.size()) {
            return EncodeResult::pass_through(std::move(payload));
        }

        ResidualSegment segment;
        segment.range = actual.range;
        segment.delta.resize(actual.data.size());
        for (size_t j = 0; j < actual.data.size(); ++j) {
            segment.delta[j] = actual.data[j] ^ pred.data[j];
        }
        residual.segments.push_back(std::move(segment));
    }

    // Step 5: EBTA validate
    auto ebta_result = detail::ebta_validate(residual);
    if (!ebta_result.valid) {
        return EncodeResult::pass_through(std::move(payload));
    }

    // Step 6: Build artifact
    Artifact artifact;
    artifact.generator = std::move(generator);
    artifact.mappings = std::move(mappings);
    artifact.predictor_state = std::move(predictor_state);
    artifact.residual = std::move(residual);

    // Generate constraints
    artifact.constraints.output_length = payload.size();
    artifact.constraints.output_hash = sha256(payload.data());

    // Generate integrity
    artifact.integrity.payload_hash = artifact.constraints.output_hash;
    artifact.integrity.version = VERSION_ID;
    artifact.integrity.encoded_at = static_cast<uint64_t>(std::time(nullptr));

    // Compute artifact hash (simplified)
    std::vector<uint8_t> artifact_content;
    artifact_content.insert(artifact_content.end(),
                           artifact.generator.base.begin(),
                           artifact.generator.base.end());
    for (const auto& seg : artifact.residual.segments) {
        artifact_content.insert(artifact_content.end(), seg.delta.begin(), seg.delta.end());
    }
    artifact.integrity.artifact_hash = sha256(artifact_content);

    return EncodeResult::encoded(std::move(artifact));
}

// =============================================================================
// Decoding Implementation
// =============================================================================

Result<Payload> vectra_decode(const Artifact& artifact) {
    // Step 1: Verify integrity
    auto verify_result = detail::verify_integrity(artifact);
    if (!verify_result) {
        return std::unexpected(verify_result.error());
    }

    // Step 2: Regenerate structure
    Structure structure = detail::regenerate_structure(artifact.generator, artifact.mappings);

    // Step 3: Build predicted variable
    VariablePart predicted;
    for (const auto& seg : artifact.residual.segments) {
        VariableSegment pred_seg;
        pred_seg.range = seg.range;
        pred_seg.data.resize(seg.delta.size(), 0);
        pred_seg.semantic_type = SemanticType::Opaque;
        predicted.segments.push_back(std::move(pred_seg));
    }

    // Step 4: Apply residual
    std::vector<std::vector<uint8_t>> residual_data;
    for (const auto& seg : artifact.residual.segments) {
        residual_data.push_back(seg.delta);
    }
    VariablePart variable = detail::reconstruct_variable(predicted, residual_data);

    // Step 5: Recompose
    auto recompose_result = detail::recompose(structure, variable);
    if (!recompose_result) {
        return std::unexpected(recompose_result.error());
    }

    // Step 6: Verify reconstruction
    auto reconstruction_verify = detail::verify_reconstruction(*recompose_result,
                                                                artifact.constraints);
    if (!reconstruction_verify) {
        return std::unexpected(reconstruction_verify.error());
    }

    return *recompose_result;
}

}  // namespace vectra
