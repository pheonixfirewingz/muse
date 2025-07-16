@echo off

REM Create certs directory if it doesn't exist
if not exist certs mkdir certs

REM Generate private key
openssl genrsa -out certs\key.pem 2048

REM Generate certificate signing request
openssl req -new -key certs\key.pem -out certs\cert.csr -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"

REM Generate self-signed certificate
openssl x509 -req -in certs\cert.csr -signkey certs\key.pem -out certs\cert.pem -days 365

REM Clean up CSR file
del certs\cert.csr

echo Self-signed certificates generated in certs\ directory
echo Certificate: certs\cert.pem
echo Private Key: certs\key.pem
echo.
echo Note: These are self-signed certificates for development only.
echo For production, use certificates from a trusted Certificate Authority.
pause 