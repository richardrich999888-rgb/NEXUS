/**
 * @file integrity.cpp
 * @brief Integrity verification implementation.
 */

#include "vectra/vectra.hpp"

namespace vectra::detail {

Result<void> verify_integrity(const Artifact& artifact) {
    // Check version
    if (artifact.integrity.version != VERSION_ID) {
        return std::unexpected(Error{
            ErrorCode::VersionMismatch,
            "Artifact version mismatch"
        });
    }

    // Recompute artifact hash
    std::vector<uint8_t> artifact_content;
    artifact_content.insert(artifact_content.end(),
                           artifact.generator.base.begin(),
                           artifact.generator.base.end());
    for (const auto& seg : artifact.residual.segments) {
        artifact_content.insert(artifact_content.end(),
                               seg.delta.begin(),
                               seg.delta.end());
    }

    Hash256 computed_hash = sha256(artifact_content);

    if (computed_hash != artifact.integrity.artifact_hash) {
        return std::unexpected(Error{
            ErrorCode::IntegrityCheckFailed,
            "Artifact hash mismatch"
        });
    }

    return {};
}

Result<void> verify_reconstruction(const Payload& payload,
                                    const ReconstructionConstraints& constraints) {
    if (payload.size() != constraints.output_length) {
        return std::unexpected(Error{
            ErrorCode::DecodeFailed,
            "Output length mismatch"
        });
    }

    Hash256 payload_hash = sha256(payload.data());
    if (payload_hash != constraints.output_hash) {
        return std::unexpected(Error{
            ErrorCode::DecodeFailed,
            "Output hash mismatch"
        });
    }

    return {};
}

}  // namespace vectra::detail
