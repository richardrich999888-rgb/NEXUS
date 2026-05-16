"""
AGP-CORE ML Module
Machine Learning, Deep Learning, and RAG capabilities
"""

from .rag_engine import (
    RAGEngine,
    ChromaVectorStore,
    FAISSVectorStore,
    EmbeddingService,
    KnowledgeChunk,
    RetrievalResult,
    rag_engine,
    add_knowledge,
    retrieve_context
)

from .deep_learning import (
    DeepLearningService,
    SklearnService,
    deep_learning_service,
    sklearn_service,
    predict_behavior,
    detect_anomaly
)

# Alias for backward compatibility if needed, though strictly speaking VectorStore is likely ChromaVectorStore
VectorStore = ChromaVectorStore

__all__ = [
    # RAG
    "RAGEngine",
    "ChromaVectorStore",
    "FAISSVectorStore",
    "VectorStore",
    "EmbeddingService",
    "KnowledgeChunk",
    "RetrievalResult",
    "rag_engine",
    "add_knowledge",
    "retrieve_context",
    
    # Deep Learning
    "DeepLearningService",
    "SklearnService",
    "deep_learning_service",
    "sklearn_service",
    "predict_behavior",
    "detect_anomaly"
]
