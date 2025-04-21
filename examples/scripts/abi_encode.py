#!/usr/bin/env python3
# Copyright (c) The Ant Group Core Contributors
# SPDX-License-Identifier: Apache-2.0
# need python3 -m pip3 install -i https://mirrors.ustc.edu.cn/pypi/simple -r requirements.txt

import sys
import argparse
from eth_abi import encode
from eth_utils import function_signature_to_4byte_selector, to_hex
from eth_utils.hexadecimal import remove_0x_prefix

def parse_signature(sig):
    # Extract function name and param types from signature like "transfer(address,uint256)"
    name_end = sig.find('(')
    if name_end == -1:
        raise ValueError("Invalid function signature")
    
    name = sig[:name_end]
    params_str = sig[name_end+1:-1]
    param_types = params_str.split(',') if params_str else []
    return name, param_types

def convert_param(param, param_type):
    # Convert string parameters to appropriate Python types
    if param_type.startswith('uint') or param_type.startswith('int'):
        return int(param)
    elif param_type == 'address':
        return remove_0x_prefix(param)
    elif param_type == 'bool':
        return param.lower() == 'true'
    elif param_type.endswith('[]'):
        # Convert comma-separated string to list for dynamic arrays
        return [convert_param(p.strip(), param_type[:-2]) for p in param.split(',')]
    return param

def main():
    parser = argparse.ArgumentParser(
        description='Encode Solidity function calls into transaction calldata',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  %(prog)s "transfer(address,uint256)" 0x1122334455667788990011223344556677889900 100
  %(prog)s "balanceOf(address)" 0x1122334455667788990011223344556677889900
''')
    
    parser.add_argument('signature', 
                      help='Solidity function signature (e.g., "transfer(address,uint256)")')
    parser.add_argument('params', 
                      nargs='*', 
                      help='Function parameters (should match the types in signature)')

    args = parser.parse_args()
    
    # Parse function signature
    func_name, param_types = parse_signature(args.signature)
    
    if len(args.params) != len(param_types):
        parser.error(f"Expected {len(param_types)} parameters for {args.signature}, got {len(args.params)}")
    
    try:
        # Convert parameters to appropriate types
        converted_params = [convert_param(p, t) for p, t in zip(args.params, param_types)]
        
        # Get function selector
        selector = function_signature_to_4byte_selector(args.signature)
        
        # Encode parameters
        encoded_params = encode(param_types, converted_params)
        
        # Combine selector with encoded parameters
        calldata = to_hex(selector + encoded_params)
        
        print(calldata)
        
    except ValueError as e:
        parser.error(f"Parameter conversion error: {str(e)}")
    except Exception as e:
        parser.error(f"Encoding error: {str(e)}")

if __name__ == '__main__':
    main()
