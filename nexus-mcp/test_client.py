"""
Test MCP Client
Verifies nexus-mcp/server.py using Stdio transport.
"""
import subprocess
import json
import os
import sys

SERVER_PATH = os.path.join(os.path.dirname(__file__), "server.py")

def send_request(proc, method, params=None, req_id=1):
    req = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params or {},
        "id": req_id
    }
    json_str = json.dumps(req)
    proc.stdin.write(json_str + "\n")
    proc.stdin.flush()
    
    response_str = proc.stdout.readline()
    return json.loads(response_str)

def run_test():
    print(f"🚀 Launching Server: {SERVER_PATH}")
    proc = subprocess.Popen(
        [sys.executable, SERVER_PATH],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr,
        text=True
    )
    
    try:
        # 1. Initialize
        print("\n--- Testing Initialize ---")
        res = send_request(proc, "initialize", req_id=1)
        print(json.dumps(res, indent=2))
        assert res["result"]["server"]["name"] == "nexus-mcp"
        
        # 2. List Tools
        print("\n--- Testing tools/list ---")
        res = send_request(proc, "tools/list", req_id=2)
        print(json.dumps(res, indent=2))
        tools = [t["name"] for t in res["result"]["tools"]]
        assert "measure_entropy" in tools
        assert "scan_threat_pattern" in tools
        
        # 3. Call measure_entropy
        print("\n--- Testing tools/call (measure_entropy) ---")
        params = {"name": "measure_entropy", "arguments": {"trace": "aaaaa", "intent": "loop"}}
        res = send_request(proc, "tools/call", params, req_id=3)
        print(json.dumps(res, indent=2))
        # Result content should be a JSON string of TIH diagnostics
        content = json.loads(res["result"]["content"][0]["text"])
        print(f"Entropy: {content.get('logic_entropy')}")
        
        # 4. Read Resource
        print("\n--- Testing resources/read (nexus://patents) ---")
        params = {"uri": "nexus://patents"}
        res = send_request(proc, "resources/read", params, req_id=4)
        # Don't print full content, just length
        content_len = len(res["result"]["contents"][0]["text"])
        print(f"Read {content_len} bytes from nexus://patents")
        assert content_len > 100
        
        print("\n✅ MCP SERVER VERIFIED SUCCESSFULLY")
        
    finally:
        proc.terminate()

if __name__ == "__main__":
    run_test()
