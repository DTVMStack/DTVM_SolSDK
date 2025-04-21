import sys
import argparse

def binary_to_hex_file(input_path, output_path):
    try:
        # Read the binary file
        with open(input_path, 'rb') as binary_file:
            # Read the entire file content
            binary_data = binary_file.read()

        # Convert binary data to hexadecimal string
        hex_string = binary_data.hex()

        # Write the hexadecimal string to a file
        with open(output_path, 'w') as hex_file:
            hex_file.write(hex_string)

        print(f"Successfully converted {input_path} to {output_path}")

    except FileNotFoundError:
        print(f"Error: File {input_path} not found")
    except PermissionError:
        print(f"Error: No permission to read {input_path} or write to {output_path}")
    except Exception as e:
        print(f"An unknown error occurred: {e}")

def main():
    # Create argument parser
    parser = argparse.ArgumentParser(description='Convert a binary file to a hexadecimal file')

    # Add input and output file arguments
    parser.add_argument('input', help='Path to the input binary file')
    parser.add_argument('output', help='Path to the output hexadecimal file')

    # Parse command line arguments
    args = parser.parse_args()

    # Call the conversion function
    binary_to_hex_file(args.input, args.output)

# Ensure the script runs only when executed directly
if __name__ == '__main__':
    main()
