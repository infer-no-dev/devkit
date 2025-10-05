#!/usr/bin/env python3
"""
Example Python Plugin for DevKit

This plugin demonstrates the plugin loading and execution system.
It can be loaded dynamically by the PluginManager.
"""

import json
import sys
from typing import Dict, Any


def initialize():
    """Initialize the plugin"""
    print("Python test plugin initializing...")
    return "initialized"


def execute(input_data: str) -> str:
    """Execute the plugin with input data"""
    print(f"Python test plugin executing with input: {input_data}")
    
    try:
        # Try to parse input as JSON
        data = json.loads(input_data)
        result = {
            "plugin": "test-plugin",
            "processed_input": data,
            "timestamp": "2024-01-01T00:00:00Z",
            "status": "success"
        }
        return json.dumps(result, indent=2)
    except json.JSONDecodeError:
        # Handle plain text input
        result = {
            "plugin": "test-plugin",
            "processed_input": input_data,
            "timestamp": "2024-01-01T00:00:00Z",
            "status": "success"
        }
        return json.dumps(result, indent=2)


def shutdown():
    """Shutdown the plugin"""
    print("Python test plugin shutting down...")
    return "shutdown"


def main():
    """Main entry point for command-line execution"""
    if len(sys.argv) < 2:
        print("Usage: python plugin.py <method> [args...]")
        sys.exit(1)
    
    method = sys.argv[1]
    
    if method == "initialize":
        result = initialize()
        print(result)
    elif method == "execute":
        if len(sys.argv) < 3:
            print("Execute method requires input data")
            sys.exit(1)
        input_data = sys.argv[2]
        result = execute(input_data)
        print(result)
    elif method == "shutdown":
        result = shutdown()
        print(result)
    else:
        print(f"Unknown method: {method}")
        sys.exit(1)


if __name__ == "__main__":
    main()