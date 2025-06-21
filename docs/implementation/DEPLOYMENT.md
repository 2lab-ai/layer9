# Layer9 Deployment Guide

Layer9 provides a comprehensive deployment system that works directly with platform APIs, eliminating the need for external CLI tools.

## Supported Platforms

- **Vercel** - Full API integration ✅
- **Netlify** - Full API integration ✅
- **AWS S3/CloudFront** - Coming soon 🚧
- **Cloudflare Pages** - Coming soon 🚧

## Quick Start

### 1. Generate Deployment Configuration

```bash
# Generate deployment config file
layer9 deploy-init

# Generate example environment file
layer9 deploy-init --env-example
```

### 2. Configure Your Deployment

Edit `layer9.deploy.toml`:

```toml
[deploy]
platform = "vercel"          # or "netlify", "aws", "cloudflare"
build_dir = "dist"           
environment = "production"   
project_name = "my-app"

[platforms.vercel]
token = "$VERCEL_TOKEN"      # From environment variable
```

### 3. Set Up Environment Variables

Create `.env` file:

```bash
VERCEL_TOKEN=your-vercel-api-token
NETLIFY_TOKEN=your-netlify-api-token

# Production secrets
DATABASE_URL=postgres://...
JWT_SECRET=your-secret-key
```

### 4. Build Your Project

```bash
layer9 build --mode production
```

### 5. Deploy

```bash
# Deploy to default platform
layer9 deploy

# Deploy to specific platform
layer9 deploy --target netlify

# Deploy to staging environment
layer9 deploy --env staging

# Force deployment without confirmation
layer9 deploy --force
```

## Deployment Commands

### Deploy
```bash
layer9 deploy [OPTIONS]

Options:
  -t, --target <PLATFORM>    Target platform [default: vercel]
  -e, --env <ENVIRONMENT>    Environment [default: production]
  -b, --build-dir <DIR>      Build directory [default: dist]
  -c, --config <FILE>        Config file [default: layer9.deploy.toml]
  -f, --force                Skip confirmation
  -v, --verbose              Show deployment logs
```

### Check Status
```bash
# Check deployment status
layer9 deploy-status <DEPLOYMENT_ID> [--platform <PLATFORM>]
```

### List Deployments
```bash
# List recent deployments
layer9 deploy-list [--platform <PLATFORM>] [--limit <N>]
```

### Rollback
```bash
# Rollback to previous deployment
layer9 deploy-rollback <DEPLOYMENT_ID> [--platform <PLATFORM>]
```

## Configuration

### Full Configuration Example

```toml
# layer9.deploy.toml

[deploy]
platform = "vercel"
build_dir = "dist"
environment = "production"
project_name = "my-layer9-app"
domain = "myapp.com"
notifications = true

# Vercel configuration
[platforms.vercel]
token = "$VERCEL_TOKEN"
team_id = "team_xxx"          # Optional
project_id = "prj_xxx"        # For existing projects
framework = "vanilla"
node_version = "18.x"

# Netlify configuration
[platforms.netlify]
token = "$NETLIFY_TOKEN"
team_id = "netlify-team"

# AWS configuration
[platforms.aws]
token = "$AWS_ACCESS_KEY_ID"
region = "us-east-1"

# Cloudflare configuration
[platforms.cloudflare]
token = "$CLOUDFLARE_API_TOKEN"
account_id = "your-account-id"

# Production environment
[environments.production]
domain = "app.myapp.com"

[environments.production.variables]
API_URL = "https://api.myapp.com"
PUBLIC_URL = "https://app.myapp.com"

[environments.production.secrets]
secrets = ["DATABASE_URL", "JWT_SECRET", "API_KEY"]

[environments.production.hooks]
pre_build = "cargo test"
post_deploy = "curl -X POST https://api.myapp.com/deploy-webhook"

# Staging environment
[environments.staging]
domain = "staging.myapp.com"
build_command = "layer9 build --mode staging"

[environments.staging.variables]
API_URL = "https://staging-api.myapp.com"
PUBLIC_URL = "https://staging.myapp.com"
DEBUG = "true"

# Preview environment
[environments.preview]
[environments.preview.variables]
API_URL = "https://preview-api.myapp.com"
DEBUG = "true"
```

## Environment Variables

Layer9 supports multiple ways to configure environment variables:

1. **`.env` files** - Local environment files
   - `.env` - Default
   - `.env.local` - Local overrides (gitignored)
   - `.env.production` - Production specific
   - `.env.staging` - Staging specific

2. **Configuration** - In `layer9.deploy.toml`
   ```toml
   [environments.production.variables]
   API_URL = "https://api.example.com"
   ```

3. **Secrets** - Sensitive values
   ```toml
   [environments.production.secrets]
   secrets = ["DATABASE_URL", "JWT_SECRET"]
   ```

## Platform-Specific Features

### Vercel

- Automatic preview deployments for branches
- Custom domains configuration
- Edge functions support
- Analytics integration

### Netlify

- Form handling
- Identity/authentication
- Split testing
- Build plugins

### AWS (Coming Soon)

- S3 static hosting
- CloudFront CDN
- Route53 DNS
- Cost estimation

### Cloudflare (Coming Soon)

- Workers integration
- KV storage
- Durable Objects
- Web Analytics

## Deployment Hooks

You can define hooks to run at different stages:

```toml
[environments.production.hooks]
pre_build = "npm test"              # Before build
post_build = "echo 'Build done'"    # After build
post_deploy = "notify-team.sh"      # After deployment
```

## Troubleshooting

### Missing API Token

If you see "VERCEL_TOKEN environment variable not set":

1. Get your API token from the platform dashboard
2. Add to `.env` file: `VERCEL_TOKEN=your-token`
3. Or export: `export VERCEL_TOKEN=your-token`

### Build Directory Not Found

If you see "Build directory not found":

1. Run `layer9 build` first
2. Check `build_dir` in configuration
3. Verify the path exists

### Deployment Failed

Check the deployment logs:
```bash
layer9 deploy --verbose
```

Common issues:
- Invalid API token
- Missing required environment variables
- Build errors
- Platform-specific limits

## Security Best Practices

1. **Never commit secrets** - Use `.env` files (gitignored)
2. **Use environment variables** - For API tokens
3. **Rotate tokens regularly** - Update platform tokens
4. **Limit token scope** - Use deployment-only tokens
5. **Audit deployments** - Review deployment history

## CI/CD Integration

### GitHub Actions

```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build
        run: layer9 build --mode production
        
      - name: Deploy
        run: layer9 deploy --force
        env:
          VERCEL_TOKEN: ${{ secrets.VERCEL_TOKEN }}
```

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - layer9 build --mode production
    - layer9 deploy --force
  variables:
    VERCEL_TOKEN: $VERCEL_TOKEN
  only:
    - main
```

## Advanced Usage

### Multi-Environment Deployment

```bash
# Deploy to different environments
layer9 deploy --env production
layer9 deploy --env staging
layer9 deploy --env preview
```

### Custom Build Directory

```bash
# Use custom build output
layer9 deploy --build-dir ./custom-dist
```

### Platform Migration

```bash
# Deploy same build to multiple platforms
layer9 deploy --target vercel
layer9 deploy --target netlify
```

## Future Features

- **Rollback UI** - Visual deployment history
- **A/B Testing** - Traffic splitting
- **Canary Deployments** - Gradual rollout
- **Cost Tracking** - Deployment cost estimates
- **Performance Monitoring** - Built-in analytics
- **Edge Functions** - Serverless at the edge