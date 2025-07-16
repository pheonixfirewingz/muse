# HTTPS Setup for Muse Server

This guide explains how to set up HTTPS support for your Muse server.

## Overview

The Muse server now supports both HTTP and HTTPS connections. When HTTPS certificates are available, the server will:

1. Start both HTTP (port 8000) and HTTPS (port 8443) servers
2. Allow clients to connect via either protocol
3. Automatically fall back to HTTP-only if certificates are missing

## Quick Setup (Development)

### 1. Generate Self-Signed Certificates

**On Linux/macOS:**
```bash
cd Server
chmod +x generate-certs.sh
./generate-certs.sh
```

**On Windows:**
```cmd
cd Server
generate-certs.bat
```

This will create:
- `certs/cert.pem` - Self-signed certificate
- `certs/key.pem` - Private key

### 2. Start the Server

The server will automatically detect the certificates and start both HTTP and HTTPS servers:

```bash
cargo run
```

You should see:
```
INFO HTTPS certificates found, starting both HTTP and HTTPS servers
INFO HTTP server listening on 127.0.0.1:8000
INFO HTTPS server listening on 127.0.0.1:8443
```

## Production Setup

### 1. Obtain SSL Certificates

For production, use certificates from a trusted Certificate Authority (CA):

- **Let's Encrypt** (free)
- **Cloudflare** (free with domain)
- **Commercial CAs** (paid)

### 2. Configure Certificate Paths

Update your `.env` file:

```env
HTTPS_CERT_PATH="/path/to/your/certificate.pem"
HTTPS_KEY_PATH="/path/to/your/private-key.pem"
HTTPS_PORT="443"  # Standard HTTPS port
```

### 3. File Permissions

Ensure the certificate files have proper permissions:

```bash
chmod 600 certs/key.pem
chmod 644 certs/cert.pem
```

## Configuration Options

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HTTPS_CERT_PATH` | `certs/cert.pem` | Path to SSL certificate |
| `HTTPS_KEY_PATH` | `certs/key.pem` | Path to private key |
| `HTTPS_PORT` | `8443` | HTTPS server port |

### Certificate Formats

The server supports:
- **PEM format** certificates and keys
- **PKCS8** and **RSA** private keys
- **Certificate chains** (multiple certificates in one file)

## Client Configuration

### Frontend (Angular)

The frontend automatically detects and uses HTTPS when available:

```typescript
// The ApiService will try HTTPS first, then fall back to HTTP
this.apiService.configure({
  host: '127.0.0.1',
  port: 8000,
  useHttps: true
});
```

### Electron App

The Electron app includes HTTPS support in the CSP:

```html
<meta http-equiv="Content-Security-Policy" 
      content="connect-src 'self' file: http://*:8000 https://*:8000;">
```

## Troubleshooting

### Certificate Issues

1. **"Failed to create TLS acceptor"**
   - Check certificate file paths
   - Verify certificate and key are valid
   - Ensure proper file permissions

2. **"TLS handshake failed"**
   - Certificate may be expired
   - Certificate doesn't match domain
   - Client doesn't trust self-signed certificate

### Port Issues

1. **"Address already in use"**
   - Change `HTTPS_PORT` in `.env`
   - Check if another service is using the port

### Development vs Production

- **Development**: Use self-signed certificates
- **Production**: Use CA-signed certificates
- **Testing**: Browsers will show security warnings for self-signed certs

## Security Notes

1. **Self-signed certificates** are for development only
2. **Private keys** should be kept secure and never shared
3. **Certificate renewal** should be automated in production
4. **HTTP fallback** ensures service availability

## Example .env Configuration

```env
SERVER_BIND="127.0.0.1:8000"
CACHE_DIR="runtime/cache"
CONTACT_EMAIL="contact@example.com"
WEBSITE_URL="127.0.0.1:8000"

# HTTPS Configuration
HTTPS_CERT_PATH="certs/cert.pem"
HTTPS_KEY_PATH="certs/key.pem"
HTTPS_PORT="8443"
```

## Testing HTTPS

1. **Start the server** with certificates
2. **Test HTTP**: `curl http://127.0.0.1:8000/api/health`
3. **Test HTTPS**: `curl -k https://127.0.0.1:8443/api/health`
4. **Frontend**: The app will automatically use HTTPS when available 