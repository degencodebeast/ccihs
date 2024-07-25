# CCIHS (Cross-Chain Interoperability Hooks for Solana)

## Overview

CCIHS is a developer toolkit designed to simplify the process of building cross-chain applications on Solana. It provides a unified, Solana-optimized API for interacting with cross-chain protocols, initially focusing on Wormhole integration.

## Features

- Unified API for cross-chain operations on Solana
- Flexible hook system for custom logic injection
- Pre-built modules for common cross-chain tasks
- Standardized error handling for cross-chain scenarios
- Solana-specific optimizations for efficient operations
- Extensible architecture supporting multiple cross-chain protocols

## Architecture

CCIHS is structured into five main layers:

1. **Core Layer**: Defines basic structures and interfaces for cross-chain operations.
2. **Protocol Layer**: Contains adapters for cross-chain protocols (initially Wormhole).
3. **API Layer**: Provides simple, consistent methods for developers.
4. **Hook Layer**: Allows insertion of custom logic at key points in the cross-chain message lifecycle.
5. **Utility Layer**: Includes tools for error handling and data formatting.

## Hook System

CCIHS implements four types of hooks:

- PreDispatch: Executed before sending a cross-chain message
- PostDispatch: Executed after sending a message
- PreExecution: Executed before processing a received message
- PostExecution: Executed after processing a received message
