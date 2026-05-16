"""
Test that all SYNTRIASS modules can be imported.
"""

import sys
import os

# Add syntriass directory to path
syntriass_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, syntriass_dir)


def test_core_imports():
    """Test core module imports"""
    try:
        sys.path.insert(0, os.path.join(syntriass_dir, 'core'))
        from inference_tap import PreviewDiffusionLoop, LatentSnapshot
        from conditioning import ConditioningInjector
        from scheduler import PreviewScheduler
        print("✓ Core modules imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Core import failed: {e}")
        return False


def test_preview_imports():
    """Test preview module imports"""
    try:
        sys.path.insert(0, os.path.join(syntriass_dir, 'preview'))
        from fast_decoder import FastPreviewDecoder
        from temporal import TemporalInterpolator
        from stream import PreviewBus
        print("✓ Preview modules imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Preview import failed: {e}")
        return False


def test_api_imports():
    """Test API module imports"""
    try:
        sys.path.insert(0, os.path.join(syntriass_dir, 'api'))
        from websocket import PreviewWebSocketServer
        from control import app
        print("✓ API modules imported successfully")
        return True
    except ImportError as e:
        print(f"✗ API import failed: {e}")
        return False


def test_patch_imports():
    """Test patch module imports"""
    try:
        sys.path.insert(0, os.path.join(syntriass_dir, 'patch'))
        from diffusers_hook import patch_pipeline
        print("✓ Patch modules imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Patch import failed: {e}")
        return False


if __name__ == "__main__":
    print("SYNTRIASS Path 6 — Import Tests")
    print("=" * 50)
    
    results = [
        test_core_imports(),
        test_preview_imports(),
        test_api_imports(),
        test_patch_imports(),
    ]
    
    print("=" * 50)
    if all(results):
        print("✓ All imports successful!")
        sys.exit(0)
    else:
        print("✗ Some imports failed")
        sys.exit(1)

