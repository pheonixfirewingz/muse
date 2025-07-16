#!/bin/bash

# Create certs directory if it doesn't exist
mkdir -p certs

# Generate private key
openssl genrsa -out certs/key.pem 2048

# Generate certificate signing request
openssl req -new -key certs/key.pem -out certs/cert.csr -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

# Generate self-signed certificate
openssl x509 -req -in certs/cert.csr -signkey certs/key.pem -out certs/cert.pem -days 365

# Clean up CSR file
rm certs/cert.csr

echo "Self-signed certificates generated in certs/ directory"
echo "Certificate: certs/cert.pem"
echo "Private Key: certs/key.pem"
echo ""
echo "Note: These are self-signed certificates for development only."
echo "For production, use certificates from a trusted Certificate Authority." 