/**
 * @file vectra.hpp
 * @brief VECTRA — Deterministic Lossless Data Volume Reduction
 *
 * VECTRA is a deterministic, lossless data reduction system for structured payloads.
 * It operates transparently beneath existing protocols and produces self-describing
 * artifacts that guarantee exact reconstruction or safe pass-through.
 *
 * Core Invariants:
 * 1. Determinism: Same input + same version → identical output
 * 2. Losslessness: decode(encode(D)) == D always
 * 3. Fail-open: Uncertainty → return original payload unchanged
 * 4. Self-describing: Artifacts contain all reconstruction information
 */

#ifndef VECTRA_HPP
#define VECTRA_HPP

#include "vectra/types.hpp"

#include <expected>
#include <string>
#include <system_error>

namespace vectra {

// =============================================================================
// Error Handling
// =============================================================================

/**
 * @brief Error codes for VECTRA operations.
 */
enum class ErrorCode {
    Success = 0,
    DecompositionFailed,
    FeeEncodingFailed,
    NsgePredictionFailed,
    EbtaValidationFailed,
    ArtifactConstructionFailed,
    IntegrityCheckFailed,
    VersionMismatch,
    DecodeFailed,
    InvalidInput
};

/**
 * @brief VECTRA error with code and message.
 */
class Error {
public:
    Error(ErrorCode code, std::string message)
        : code_(code), message_(std::move(message)) {}

    [[nodiscard]] ErrorCode code() const noexcept { return code_; }
    [[nodiscard]] const std::string& message() const noexcept { return message_; }

private:
    ErrorCode code_;
    std::string message_;
};

/// Result type for VECTRA operations.
template <typename T>
using Result = std::expected<T, Error>;

// =============================================================================
// Core API
// =============================================================================

/**
 * @brief Encode a payload using VECTRA.
 *
 * @param payload The payload to encode
 * @return EncodeResult containing either an artifact or the original payload
 *
 * Guarantees:
 * - Determinism: same input → same output
 * - Losslessness: decode(encode(D)) == D
 * - Fail-open: uncertainty → return original
 */
[[nodiscard]] EncodeResult vectra_encode(Payload payload);

/**
 * @brief Decode an artifact back to the original payload.
 *
 * @param artifact The artifact to decode
 * @return Result containing the decoded payload or an error
 *
 * Guarantees:
 * - Determinism: same artifact → same payload
 * - Losslessness: output matches original exactly
 */
[[nodiscard]] Result<Payload> vectra_decode(const Artifact& artifact);

/**
 * @brief Check if this library can decode an artifact.
 *
 * @param artifact The artifact to check
 * @return true if version matches, false otherwise
 */
[[nodiscard]] bool can_decode(const Artifact& artifact) noexcept;

// =============================================================================
// Utilities
// =============================================================================

/**
 * @brief Compute SHA-256 hash.
 *
 * @param data Input data
 * @return 32-byte hash
 */
[[nodiscard]] Hash256 sha256(const std::vector<uint8_t>& data);

/**
 * @brief Compute Shannon entropy of byte sequence.
 *
 * @param data Input bytes
 * @return Entropy in bits (0.0 to 8.0)
 */
[[nodiscard]] double compute_byte_entropy(const std::vector<uint8_t>& data);

/**
 * @brief Estimate artifact size in bytes.
 *
 * @param artifact The artifact to measure
 * @return Estimated size
 */
[[nodiscard]] size_t estimate_artifact_size(const Artifact& artifact);

/**
 * @brief Compute compression ratio.
 *
 * @param original_size Original payload size
 * @param artifact The encoded artifact
 * @return Ratio (>1.0 indicates compression benefit)
 */
[[nodiscard]] double compression_ratio(size_t original_size, const Artifact& artifact);

// =============================================================================
// Internal Components (for testing/debugging)
// =============================================================================

namespace detail {

/// Decomposition result.
struct DecompositionResult {
    Structure structure;
    VariablePart variable;
};

/// FEE encoding result.
struct FeeResult {
    Generator generator;
    MappingSet mappings;
};

/// NSGE prediction result.
struct NsgeResult {
    VariablePart predicted;
    PredictorState state;
};

/// EBTA validation result.
struct EbtaResult {
    bool valid;
    double entropy;
    double max_entropy;
};

/// Decompose payload into structure and variable parts.
[[nodiscard]] Result<DecompositionResult> decompose(const Payload& payload);

/// Encode structure using FEE.
[[nodiscard]] Result<FeeResult> fee_encode(const Structure& structure);

/// Predict variable component using NSGE.
[[nodiscard]] Result<NsgeResult> nsge_predict(const VariablePart& variable);

/// Validate residual using EBTA.
[[nodiscard]] EbtaResult ebta_validate(const Residual& residual, double h_max = H_MAX);

/// Regenerate structure from generator and mappings.
[[nodiscard]] Structure regenerate_structure(const Generator& generator,
                                              const MappingSet& mappings);

/// Reconstruct variable from prediction and residual.
[[nodiscard]] VariablePart reconstruct_variable(const VariablePart& predicted,
                                                 const std::vector<std::vector<uint8_t>>& residual_data);

/// Recompose payload from structure and variable.
[[nodiscard]] Result<Payload> recompose(const Structure& structure,
                                         const VariablePart& variable);

/// Verify artifact integrity.
[[nodiscard]] Result<void> verify_integrity(const Artifact& artifact);

/// Verify reconstruction matches constraints.
[[nodiscard]] Result<void> verify_reconstruction(const Payload& payload,
                                                  const ReconstructionConstraints& constraints);

}  // namespace detail

}  // namespace vectra

#endif  // VECTRA_HPP
