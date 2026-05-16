"""
AGP-OS LLM Provider Abstraction
Unified interface for multiple LLM providers (OpenAI, Anthropic, Google)
"""

from abc import ABC, abstractmethod
from typing import AsyncIterator, List, Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum
import time

class TokenChunk:
    """A chunk of tokens from streaming response"""
    def __init__(self, content: str, tokens: int = 0):
        self.content = content
        self.tokens = tokens

@dataclass
class CompletionResult:
    """Result from LLM completion"""
    content: str
    tokens_used: int
    duration: float
    model: str
    finish_reason: str = "stop"

class LLMProvider(ABC):
    """Abstract base class for LLM providers"""
    
    @abstractmethod
    async def stream_completion(
        self,
        messages: List[Dict[str, str]],
        model: str,
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> AsyncIterator[TokenChunk]:
        """Stream completion tokens"""
        pass
    
    @abstractmethod
    async def complete(
        self,
        messages: List[Dict[str, str]],
        model: str,
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> CompletionResult:
        """Non-streaming completion"""
        pass
    
    @abstractmethod
    def count_tokens(self, text: str, model: str = "") -> int:
        """Count tokens in text"""
        pass
    
    @abstractmethod
    def get_context_limit(self, model: str) -> int:
        """Get context window limit for model"""
        pass


class OpenAIProvider(LLMProvider):
    """OpenAI GPT provider"""
    
    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key
        self._client = None
        self._tiktoken = None
        
        # Try to import OpenAI
        try:
            import openai
            self._openai = openai
            if api_key:
                self._client = openai.AsyncOpenAI(api_key=api_key)
        except ImportError:
            self._openai = None
            
        # Try to import tiktoken for token counting
        try:
            import tiktoken
            self._tiktoken = tiktoken
        except ImportError:
            pass
    
    async def stream_completion(
        self,
        messages: List[Dict[str, str]],
        model: str = "gpt-4",
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> AsyncIterator[TokenChunk]:
        """Stream completion from OpenAI"""
        if not self._client:
            raise RuntimeError("OpenAI client not initialized. Install 'openai' package.")
        
        stream = await self._client.chat.completions.create(
            model=model,
            messages=messages,
            max_tokens=max_tokens,
            temperature=temperature,
            stream=True
        )
        
        async for chunk in stream:
            if chunk.choices[0].delta.content:
                content = chunk.choices[0].delta.content
                # Approximate token count (1 token ≈ 4 chars)
                tokens = len(content) // 4
                yield TokenChunk(content, tokens)
    
    async def complete(
        self,
        messages: List[Dict[str, str]],
        model: str = "gpt-4",
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> CompletionResult:
        """Non-streaming completion"""
        if not self._client:
            raise RuntimeError("OpenAI client not initialized")
        
        start = time.time()
        
        response = await self._client.chat.completions.create(
            model=model,
            messages=messages,
            max_tokens=max_tokens,
            temperature=temperature
        )
        
        duration = time.time() - start
        
        return CompletionResult(
            content=response.choices[0].message.content,
            tokens_used=response.usage.total_tokens,
            duration=duration,
            model=model,
            finish_reason=response.choices[0].finish_reason
        )
    
    def count_tokens(self, text: str, model: str = "gpt-4") -> int:
        """Count tokens using tiktoken"""
        if not self._tiktoken:
            # Fallback: rough approximation
            return len(text) // 4
        
        try:
            encoding = self._tiktoken.encoding_for_model(model)
            return len(encoding.encode(text))
        except:
            return len(text) // 4
    
    def get_context_limit(self, model: str) -> int:
        """Get context window size"""
        limits = {
            "gpt-4": 8192,
            "gpt-4-32k": 32768,
            "gpt-4-turbo": 128000,
            "gpt-4o": 128000,
            "gpt-3.5-turbo": 16385,
            "gpt-3.5-turbo-16k": 16385
        }
        return limits.get(model, 8192)


class AnthropicProvider(LLMProvider):
    """Anthropic Claude provider"""
    
    def __init__(self, api_key: Optional[str] = None):
        self.api_key = api_key
        self._client = None
        
        try:
            import anthropic
            self._anthropic = anthropic
            if api_key:
                self._client = anthropic.AsyncAnthropic(api_key=api_key)
        except ImportError:
            self._anthropic = None
    
    async def stream_completion(
        self,
        messages: List[Dict[str, str]],
        model: str = "claude-3-5-sonnet-20241022",
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> AsyncIterator[TokenChunk]:
        """Stream completion from Anthropic"""
        if not self._client:
            raise RuntimeError("Anthropic client not initialized. Install 'anthropic' package.")
        
        # Convert messages format (Anthropic uses different structure)
        system = None
        claude_messages = []
        for msg in messages:
            if msg["role"] == "system":
                system = msg["content"]
            else:
                claude_messages.append(msg)
        
        stream = await self._client.messages.create(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            system=system,
            messages=claude_messages,
            stream=True
        )
        
        async for event in stream:
            if event.type == "content_block_delta":
                content = event.delta.text
                tokens = len(content) // 4
                yield TokenChunk(content, tokens)
    
    async def complete(
        self,
        messages: List[Dict[str, str]],
        model: str = "claude-3-5-sonnet-20241022",
        max_tokens: int = 1000,
        temperature: float = 0.7
    ) -> CompletionResult:
        """Non-streaming completion"""
        if not self._client:
            raise RuntimeError("Anthropic client not initialized")
        
        start = time.time()
        
        # Convert messages
        system = None
        claude_messages = []
        for msg in messages:
            if msg["role"] == "system":
                system = msg["content"]
            else:
                claude_messages.append(msg)
        
        response = await self._client.messages.create(
            model=model,
            max_tokens=max_tokens,
            temperature=temperature,
            system=system,
            messages=claude_messages
        )
        
        duration = time.time() - start
        
        return CompletionResult(
            content=response.content[0].text,
            tokens_used=response.usage.input_tokens + response.usage.output_tokens,
            duration=duration,
            model=model,
            finish_reason=response.stop_reason
        )
    
    def count_tokens(self, text: str, model: str = "") -> int:
        """Count tokens (Anthropic uses similar tokenization to GPT)"""
        # Anthropic doesn't provide a public tokenizer
        # Rough approximation: 1 token ≈ 4 characters
        return len(text) // 4
    
    def get_context_limit(self, model: str) -> int:
        """Get context window size"""
        limits = {
            "claude-3-5-sonnet-20241022": 200000,
            "claude-3-opus-20240229": 200000,
            "claude-3-sonnet-20240229": 200000,
            "claude-3-haiku-20240307": 200000
        }
        return limits.get(model, 200000)


# Provider registry
_providers: Dict[str, LLMProvider] = {}

def register_provider(name: str, provider: LLMProvider):
    """Register an LLM provider"""
    _providers[name] = provider

def get_provider(name: str) -> Optional[LLMProvider]:
    """Get a registered provider"""
    return _providers.get(name)

def list_providers() -> List[str]:
    """List all registered providers"""
    return list(_providers.keys())
