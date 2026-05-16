"""
AGP-CORE RAG Engine
Retrieval-Augmented Generation for context-aware agent intelligence
"""

import uuid
import hashlib
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from dataclasses import dataclass, field

# Try to import RAG dependencies
try:
    import chromadb
    from chromadb.config import Settings as ChromaSettings
    CHROMADB_AVAILABLE = True
except ImportError:
    CHROMADB_AVAILABLE = False

try:
    from sentence_transformers import SentenceTransformer
    SENTENCE_TRANSFORMERS_AVAILABLE = True
except ImportError:
    SENTENCE_TRANSFORMERS_AVAILABLE = False

try:
    import faiss
    import numpy as np
    FAISS_AVAILABLE = True
except ImportError:
    FAISS_AVAILABLE = False


@dataclass
class KnowledgeChunk:
    """A chunk of knowledge for RAG"""
    id: str
    content: str
    metadata: Dict[str, Any]
    embedding: Optional[List[float]] = None
    created_at: datetime = field(default_factory=datetime.utcnow)


@dataclass
class RetrievalResult:
    """Result from RAG retrieval"""
    chunks: List[KnowledgeChunk]
    scores: List[float]
    query: str
    total_results: int


class EmbeddingService:
    """
    Generates embeddings for text using sentence-transformers
    """
    
    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        self.model_name = model_name
        self._model = None
        self._dimension = 384  # Default for MiniLM
    
    @property
    def model(self):
        if self._model is None and SENTENCE_TRANSFORMERS_AVAILABLE:
            self._model = SentenceTransformer(self.model_name)
            self._dimension = self._model.get_sentence_embedding_dimension()
        return self._model
    
    @property
    def dimension(self) -> int:
        return self._dimension
    
    def embed_text(self, text: str) -> List[float]:
        """Generate embedding for single text"""
        if self.model is None:
            # Fallback: simple hash-based embedding for testing
            return self._fallback_embedding(text)
        
        embedding = self.model.encode(text, convert_to_numpy=True)
        return embedding.tolist()
    
    def embed_batch(self, texts: List[str]) -> List[List[float]]:
        """Generate embeddings for multiple texts"""
        if self.model is None:
            return [self._fallback_embedding(t) for t in texts]
        
        embeddings = self.model.encode(texts, convert_to_numpy=True)
        return embeddings.tolist()
    
    def _fallback_embedding(self, text: str, dim: int = 384) -> List[float]:
        """Simple fallback embedding using hash"""
        h = hashlib.sha256(text.encode()).hexdigest()
        # Convert hash to floats
        embedding = []
        for i in range(0, min(len(h), dim * 2), 2):
            val = int(h[i:i+2], 16) / 255.0 - 0.5
            embedding.append(val)
        # Pad if needed
        while len(embedding) < dim:
            embedding.append(0.0)
        return embedding[:dim]


class FAISSVectorStore:
    """High-performance local vector store using FAISS"""
    
    def __init__(self, dimension: int = 384):
        self.dimension = dimension
        self.index = None
        self.metadata = {}  # Map ID -> Metadata
        self.id_map = {}    # Map FAISS ID -> Real ID
        self.current_idx = 0
        
        if FAISS_AVAILABLE:
            self.index = faiss.IndexFlatL2(dimension)
    
    def add(self, embeddings: List[List[float]], metadatas: List[Dict]):
        if not FAISS_AVAILABLE or not self.index:
            return
            
        vectors = np.array(embeddings).astype('float32')
        self.index.add(vectors)
        
        for i, meta in enumerate(metadatas):
            faiss_id = self.current_idx + i
            real_id = meta.get("id", str(uuid.uuid4()))
            
            self.id_map[faiss_id] = real_id
            self.metadata[real_id] = meta
            
        self.current_idx += len(embeddings)
        
    def search(self, query_vector: List[float], k: int = 5, filter_fn: Optional[callable] = None) -> List[Dict]:
        if not FAISS_AVAILABLE or not self.index or self.index.ntotal == 0:
            return []
            
        vector = np.array([query_vector]).astype('float32')
        distances, indices = self.index.search(vector, k * 3) # Fetch more for filtering
        
        results = []
        for i, idx in enumerate(indices[0]):
            if idx == -1: continue
            
            real_id = self.id_map.get(idx)
            if not real_id: continue
            
            meta = self.metadata.get(real_id)
            if not meta: continue
            
            # Apply filter if provided (e.g., hormone state match)
            if filter_fn and not filter_fn(meta):
                continue
                
            results.append({
                "id": real_id,
                "metadata": meta,
                "score": float(1.0 / (1.0 + distances[0][i])) # Convert distance to score
            })
            
            if len(results) >= k:
                break
                
        return results

    def count(self) -> int:
        return self.index.ntotal if self.index else 0


class ChromaVectorStore:
    """Persistent vector store using ChromaDB"""
    
    def __init__(self, collection_name: str = "agp_knowledge"):
        self.client = None
        self.collection = None
        if CHROMADB_AVAILABLE:
            try:
                self.client = chromadb.Client()
                self.collection = self.client.get_or_create_collection(
                    name=collection_name,
                    metadata={"hnsw:space": "cosine"}
                )
            except Exception as e:
                print(f"ChromaDB Init Error: {e}")
    
    def add(self, content: str, embedding: List[float], metadata: Dict) -> str:
        if not self.collection:
            return str(uuid.uuid4())
            
        doc_id = str(uuid.uuid4())
        self.collection.add(
            ids=[doc_id],
            embeddings=[embedding],
            documents=[content],
            metadatas=[metadata]
        )
        return doc_id
        
    def search(self, query_embedding: List[float], k: int = 5) -> List[Dict]:
        if not self.collection:
            return []
            
        results = self.collection.query(
            query_embeddings=[query_embedding],
            n_results=k
        )
        
        output = []
        if results["documents"]:
            for i, doc in enumerate(results["documents"][0]):
                output.append({
                    "document": doc,
                    "metadata": results["metadatas"][0][i] if results["metadatas"] else {}
                })
        return output
        
    def count(self) -> int:
        return self.collection.count() if self.collection else 0


class RAGEngine:
    """
    RAG Engine with Hybrid Storage (ChromaDB + FAISS)
    """
    
    def __init__(self):
        self.embedding_service = EmbeddingService()
        self.chroma_store = ChromaVectorStore()
        self.faiss_store = FAISSVectorStore()
        
    def add_knowledge(self, text: str, category: str = "general", metadata: Dict = None) -> str:
        """Add knowledge to both stores"""
        meta = metadata or {}
        meta.update({
            "text": text,
            "category": category,
            "timestamp": datetime.now().isoformat()
        })
        
        # Determine embedding once
        embedding = self.embedding_service.embed_text(text)
        
        # 1. Add to ChromaDB (Persistent)
        doc_id = self.chroma_store.add(text, embedding, meta)
        meta["id"] = doc_id
        
        # 2. Add to FAISS (Fast In-Memory)
        if embedding:
            self.faiss_store.add([embedding], [meta])
            
        return doc_id
        
    def retrieve_context(
        self, 
        query: str, 
        limit: int = 5, 
        endocrine_state: Optional[Dict] = None
    ) -> List[str]:
        """
        Retrieve context with hormone-based filtering.
        """
        query_embedding = self.embedding_service.embed_text(query)
        if not query_embedding:
            return []

        # Filter function based on endocrine state
        def hormone_filter(meta: Dict) -> bool:
            if not endocrine_state:
                return True
                
            # If agent is stressed (High Cortisol), prefer 'safe' or 'proven' knowledge
            if endocrine_state.get("cortisol", 0.5) > 0.7:
                return meta.get("category") in ["security", "protocol", "error_handling"]
                
            # If agent is creative (High Dopamine), prefer 'novel' or 'creative' knowledge
            if endocrine_state.get("dopamine", 0.5) > 0.7:
                return meta.get("category") in ["innovation", "strategy", "creative"]
                
            return True

        # Search FAISS first
        results = self.faiss_store.search(
            query_embedding, 
            k=limit, 
            filter_fn=hormone_filter
        )
        
        if results:
            return [r['metadata']['text'] for r in results]
            
        # Fallback to Chroma if FAISS empty/fails
        chroma_results = self.chroma_store.search(query_embedding, k=limit)
        return [res['document'] for res in chroma_results]

    def stats(self) -> Dict:
        return {
            "chroma_count": self.chroma_store.count(),
            "faiss_count": self.faiss_store.count(),
            "chroma_available": CHROMADB_AVAILABLE,
            "faiss_available": FAISS_AVAILABLE
        }

# Singleton instance
rag_engine = RAGEngine()

def add_knowledge(text: str, category: str = "general", metadata: Dict = None) -> str:
    return rag_engine.add_knowledge(text, category, metadata)

def retrieve_context(query: str, limit: int = 5, endocrine_state: Optional[Dict] = None) -> List[str]:
    return rag_engine.retrieve_context(query, limit, endocrine_state)
