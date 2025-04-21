#!/bin/bash
set -e

forge install --no-commit foundry-rs/forge-std
forge install --no-commit OpenZeppelin/openzeppelin-contracts
forge install --no-commit OpenZeppelin/openzeppelin-contracts-upgradeable
