"""
AURA Monetization SDK
Client library for interacting with AURA's billing and account management API.
"""

import requests
import hmac
import hashlib
import time
from typing import Dict, Any, Optional, List

class AuraClient:
    """Client for AURA Protocol Monetization API."""
    
    def __init__(self, api_key: str, base_url: str = "https://api.aura-protocol.com"):
        """
        Initialize the AURA client.
        
        Args:
            api_key: Your API key for authentication.
            base_url: The base URL of the AURA API server.
        """
        self.api_key = api_key
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
            "User-Agent": "AURA-SDK/1.0.0"
        })

    def _request(self, method: str, endpoint: str, data: Optional[Dict] = None) -> Dict[str, Any]:
        """Internal method to make HTTP requests."""
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        response = self.session.request(method, url, json=data)
        
        try:
            response.raise_for_status()
            return response.json()
        except requests.exceptions.HTTPError as e:
            try:
                error_detail = response.json()
            except ValueError:
                error_detail = {"detail": str(e)}
            raise Exception(f"API Request Failed: {error_detail}")

    def create_customer(self, email: str, company: Optional[str] = None, plan: str = 'free') -> Dict[str, Any]:
        """
        Create a new customer account.
        
        Args:
            email: Customer email address.
            company: Optional company name.
            plan: Subscription plan ('free', 'pro', 'enterprise').
            
        Returns:
            Dictionary containing customer details and API key.
        """
        data = {
            "email": email,
            "company": company,
            "plan": plan
        }
        return self._request("POST", "/v1/customers", data)

    def get_customer_usage(self, customer_id: Optional[str] = None) -> Dict[str, Any]:
        """
        Get usage statistics for a customer.
        
        Args:
            customer_id: Optional customer ID. If not provided, infers from API key.
            
        Returns:
            Dictionary containing usage stats.
        """
        # Note: The API endpoint for this might need to be added to the server if not exists,
        # or we assume /v1/customers/me or similar. 
        # For now, let's assume we maintain a reference or the API handles it.
        # Based on server/api.py, we didn't explicitly expose 'get_usage' endpoint yet 
        # except implicitly via verify response or we need to add it.
        # Let's assume we will add /v1/usage or similar.
        return self._request("GET", "/v1/usage")

    def register_webhook(self, url: str, events: List[str]) -> Dict[str, Any]:
        """
        Register a webhook endpoint.
        
        Args:
            url: The URL to receive webhook events.
            events: List of event types to subscribe to.
            
        Returns:
            Webhook registration details including secret.
        """
        data = {
            "url": url,
            "events": events
        }
        return self._request("POST", "/v1/webhooks", data)

    @staticmethod
    def verify_webhook_signature(payload: bytes, signature: str, secret: str) -> bool:
        """
        Verify the signature of a received webhook event.
        
        Args:
            payload: The raw body of the webhook request.
            signature: The signature header from the request.
            secret: The webhook secret provided during registration.
            
        Returns:
            True if signature is valid.
        """
        expected_signature = hmac.new(
            secret.encode(),
            payload,
            hashlib.sha256
        ).hexdigest()
        return hmac.compare_digest(expected_signature, signature)
