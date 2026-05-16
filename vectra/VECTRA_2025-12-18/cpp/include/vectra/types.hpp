/**
 * @file types.hpp
 * @brief Core type definitions for VECTRA.
 *
 * These types map directly to the formal specification:
 * - Payload: D ∈ 𝒟
 * - Artifact: A ∈ 𝒜
 * - Structure: S (stable structural components)
 * - VariablePart: V (time-evolving components)
 */

#ifndef VECTRA_TYPES_HPP
#define VECTRA_TYPES_HPP

#include <array>
#include <cstdint>
#include <optional>
#include <string>
#include <variant>
#include <vector>

namespace vectra {

/// System version identifier. All artifacts are version-locked.
constexpr uint64_t VERSION_ID = 0x0001'0000'0000'0001ULL;

/// Maximum allowed Shannon entropy for residuals (H_MAX from spec §6).
constexpr double H_MAX = 4.0;

/// SHA-256 hash type (32 bytes).
using Hash256 = std::array<uint8_t, 32>;

/**
 * @brief Byte range in original payload.
 */
struct ByteRange {
    size_t start{0};
    size_t end{0};

    [[nodiscard]] size_t length() const noexcept { return end - start; }

    bool operator==(const ByteRange& other) const noexcept {
        return start == other.start && end == other.end;
    }
};

/**
 * @brief Schema identifier for typed payload interpretation.
 */
struct SchemaId {
    std::string namespace_name;
    std::string name;
    std::tuple<uint16_t, uint16_t, uint16_t> version;
};

/**
 * @brief Raw payload bytes. Represents D ∈ 𝒟.
 */
class Payload {
public:
    Payload() = default;
    explicit Payload(std::vector<uint8_t> data) : data_(std::move(data)) {}
    Payload(const uint8_t* data, size_t size) : data_(data, data + size) {}

    [[nodiscard]] const std::vector<uint8_t>& data() const noexcept { return data_; }
    [[nodiscard]] size_t size() const noexcept { return data_.size(); }
    [[nodiscard]] bool empty() const noexcept { return data_.empty(); }

    [[nodiscard]] const std::optional<SchemaId>& schema_id() const noexcept {
        return schema_id_;
    }
    void set_schema_id(SchemaId id) { schema_id_ = std::move(id); }

    bool operator==(const Payload& other) const noexcept {
        return data_ == other.data_;
    }

private:
    std::vector<uint8_t> data_;
    std::optional<SchemaId> schema_id_;
};

/**
 * @brief Semantic type hints for variable data.
 */
enum class SemanticType {
    Counter,
    Timestamp,
    Metric,
    Identifier,
    Opaque
};

/**
 * @brief A single level in the structural hierarchy.
 */
struct StructureLevel {
    uint64_t pattern_id{0};
    std::vector<size_t> children;
    std::vector<uint8_t> literals;
};

/**
 * @brief Structural component extracted from payload (S from spec §3).
 */
struct Structure {
    std::vector<StructureLevel> levels;
    std::vector<ByteRange> byte_ranges;
};

/**
 * @brief A segment of variable data.
 */
struct VariableSegment {
    ByteRange range;
    std::vector<uint8_t> data;
    SemanticType semantic_type{SemanticType::Opaque};
};

/**
 * @brief Variable component extracted from payload (V from spec §3).
 */
struct VariablePart {
    std::vector<VariableSegment> segments;
};

/**
 * @brief Specification for how a pattern repeats.
 */
struct RepetitionSpec {
    uint32_t count{0};
    uint32_t stride{0};
};

/**
 * @brief Structural generator produced by FEE (G from spec §4).
 */
struct Generator {
    std::vector<uint8_t> base;
    RepetitionSpec repetition;
};

/**
 * @brief Transformation applied by a mapping.
 */
enum class MappingTransform {
    Identity,
    Offset,
    Concat
};

/**
 * @brief Recursive mapping function (φ from spec §4).
 */
struct Mapping {
    size_t from_level{0};
    size_t to_level{0};
    MappingTransform transform{MappingTransform::Identity};
    std::variant<int32_t, std::vector<size_t>> transform_param;
};

/**
 * @brief Set of mappings Φ = {φ₀, φ₁, ..., φₖ} from spec §4.
 */
struct MappingSet {
    std::vector<Mapping> mappings;
};

/**
 * @brief Predictor model parameters.
 */
struct PredictorParameters {
    std::vector<int64_t> counter_state;
    int64_t timestamp_base{0};
    int64_t timestamp_delta{0};
    int64_t metric_mean{0};  // Fixed-point, scale factor 1000
    int64_t metric_variance{0};
};

/**
 * @brief Predictor state (Θ from spec §5).
 */
struct PredictorState {
    uint64_t version{VERSION_ID};
    PredictorParameters parameters;
};

/**
 * @brief Residual for a single variable segment.
 */
struct ResidualSegment {
    ByteRange range;
    std::vector<uint8_t> delta;
};

/**
 * @brief Residual Δ = V - V̂ from spec §5.
 */
struct Residual {
    std::vector<ResidualSegment> segments;
};

/**
 * @brief Integrity metadata (I from spec §7).
 */
struct IntegrityMeta {
    Hash256 payload_hash{};
    Hash256 artifact_hash{};
    uint64_t version{VERSION_ID};
    uint64_t encoded_at{0};
};

/**
 * @brief Reconstruction constraints (C from spec §7).
 */
struct ReconstructionConstraints {
    size_t output_length{0};
    Hash256 output_hash{};
};

/**
 * @brief Complete VECTRA artifact (A from spec §7).
 */
struct Artifact {
    Generator generator;
    MappingSet mappings;
    PredictorState predictor_state;
    Residual residual;
    ReconstructionConstraints constraints;
    IntegrityMeta integrity;

    /// Serialize artifact to bytes (deterministic).
    [[nodiscard]] std::vector<uint8_t> to_bytes() const;

    /// Deserialize artifact from bytes.
    [[nodiscard]] static Artifact from_bytes(const std::vector<uint8_t>& data);
};

/**
 * @brief Result of encoding: either an artifact or the original payload.
 */
class EncodeResult {
public:
    /// Create successful encode result.
    static EncodeResult encoded(Artifact artifact) {
        EncodeResult result;
        result.artifact_ = std::move(artifact);
        return result;
    }

    /// Create pass-through result.
    static EncodeResult pass_through(Payload payload) {
        EncodeResult result;
        result.payload_ = std::move(payload);
        return result;
    }

    [[nodiscard]] bool is_encoded() const noexcept { return artifact_.has_value(); }
    [[nodiscard]] bool is_pass_through() const noexcept { return payload_.has_value(); }

    [[nodiscard]] const Artifact& artifact() const { return artifact_.value(); }
    [[nodiscard]] const Payload& payload() const { return payload_.value(); }

private:
    std::optional<Artifact> artifact_;
    std::optional<Payload> payload_;
};

}  // namespace vectra

#endif  // VECTRA_TYPES_HPP
