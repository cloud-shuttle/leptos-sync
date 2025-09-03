# Deployment & Operations Guide for Leptos-Sync

## Overview

**Last Updated:** September 3rd, 2025  
**Target Environment:** Production-grade deployment with enterprise operations  
**Deployment Model:** Multi-environment with progressive deployment  
**Operations Focus:** Reliability, performance, and scalability

This document outlines the comprehensive deployment and operations strategy for Leptos-Sync, ensuring reliable production deployment and efficient operational management.

## Deployment Architecture

### 1. **Multi-Environment Strategy**

**Environment Hierarchy:**
```
┌─────────────────────────────────────────────────────┐
│                Production                           │ ← Live user traffic
├─────────────────────────────────────────────────────┤
│                Staging                             │ ← Pre-production testing
├─────────────────────────────────────────────────────┤
│                Development                         │ ← Feature development
├─────────────────────────────────────────────────────┤
│                Local                                │ ← Developer environment
└─────────────────────────────────────────────────────┘
```

**Environment Configuration:**
```rust
// Environment-specific configuration
pub struct EnvironmentConfig {
    pub environment: Environment,
    pub database_url: String,
    pub redis_url: String,
    pub sync_endpoint: String,
    pub log_level: LogLevel,
    pub feature_flags: FeatureFlags,
    pub security_settings: SecuritySettings,
}

pub enum Environment {
    Local,
    Development,
    Staging,
    Production,
}

impl EnvironmentConfig {
    pub fn load() -> Result<Self, Error> {
        let environment = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "local".to_string())
            .parse()?;
        
        let config = match environment {
            Environment::Local => Self::local(),
            Environment::Development => Self::development(),
            Environment::Staging => Self::staging(),
            Environment::Production => Self::production(),
        };
        
        Ok(config)
    }
}
```

### 2. **Infrastructure as Code**

**Terraform Configuration:**
```hcl
# infrastructure/main.tf
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.0"
    }
  }
}

# VPC and networking
module "vpc" {
  source = "./modules/vpc"
  
  environment = var.environment
  vpc_cidr   = var.vpc_cidr
  azs        = var.availability_zones
}

# EKS cluster
module "eks" {
  source = "./modules/eks"
  
  cluster_name    = "leptos-sync-${var.environment}"
  cluster_version = "1.28"
  vpc_id         = module.vpc.vpc_id
  subnet_ids     = module.vpc.private_subnet_ids
}

# RDS database
module "rds" {
  source = "./modules/rds"
  
  environment     = var.environment
  vpc_id         = module.vpc.vpc_id
  subnet_ids     = module.vpc.database_subnet_ids
  
  instance_class = var.environment == "production" ? "db.r6i.xlarge" : "db.t3.medium"
  allocated_storage = var.environment == "production" ? 100 : 20
}
```

## CI/CD Pipeline

### 1. **GitHub Actions Workflow**

**Complete CI/CD Pipeline:**
```yaml
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  release:
    types: [ published ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # Quality Assurance
  quality:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
        override: true
    
    - name: Install dependencies
      run: |
        cargo install wasm-pack
        cargo install cargo-audit
        cargo install cargo-tarpaulin
    
    - name: Run tests
      run: |
        cargo test --all-features
        wasm-pack test --node --headless
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Security audit
      run: cargo audit
    
    - name: Generate coverage
      run: cargo tarpaulin --out html --output-dir coverage

  # Build and Package
  build:
    needs: quality
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, wasm32-unknown-unknown]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
        override: true
    
    - name: Build
      run: |
        if [ "${{ matrix.target }}" = "wasm32-unknown-unknown" ]; then
          wasm-pack build --target web
        else
          cargo build --release --target ${{ matrix.target }}
        fi
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: build-${{ matrix.target }}
        path: |
          target/${{ matrix.target }}/release/
          pkg/

  # Deploy to Staging
  deploy-staging:
    needs: [quality, build]
    if: github.ref == 'refs/heads/develop'
    runs-on: ubuntu-latest
    environment: staging
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Configure AWS credentials
      uses: aws-actions/configure-aws-credentials@v4
      with:
        aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
        aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        aws-region: us-west-2
    
    - name: Update kubeconfig
      run: aws eks update-kubeconfig --name leptos-sync-staging --region us-west-2
    
    - name: Deploy to staging
      run: |
        kubectl apply -f k8s/staging/
        kubectl rollout restart deployment/leptos-sync-api -n leptos-sync

  # Deploy to Production
  deploy-production:
    needs: [quality, build]
    if: github.event_name == 'release'
    runs-on: ubuntu-latest
    environment: production
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Configure AWS credentials
      uses: aws-actions/configure-aws-credentials@v4
      with:
        aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
        aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
        aws-region: us-west-2
    
    - name: Update kubeconfig
      run: aws eks update-kubeconfig --name leptos-sync-production --region us-west-2
    
    - name: Deploy to production
      run: |
        kubectl apply -f k8s/production/
        kubectl rollout restart deployment/leptos-sync-api -n leptos-sync
```

## Kubernetes Deployment

### 1. **Kubernetes Manifests**

**Core Deployment:**
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: leptos-sync-api
  namespace: leptos-sync
spec:
  replicas: 3
  selector:
    matchLabels:
      app: leptos-sync-api
  template:
    metadata:
      labels:
        app: leptos-sync-api
    spec:
      containers:
      - name: leptos-sync-api
        image: leptos-sync/api:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: leptos-sync-config
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL

---
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: leptos-sync-api-service
  namespace: leptos-sync
spec:
  selector:
    app: leptos-sync-api
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP

---
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: leptos-sync-ingress
  namespace: leptos-sync
  annotations:
    kubernetes.io/ingress.class: "nginx"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - api.leptos-sync.com
    - sync.leptos-sync.com
    secretName: leptos-sync-tls
  rules:
  - host: api.leptos-sync.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: leptos-sync-api-service
            port:
              number: 80
```

### 2. **Auto-scaling Configuration**

**Horizontal Pod Autoscaler:**
```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: leptos-sync-api-hpa
  namespace: leptos-sync
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: leptos-sync-api
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
```

## Monitoring & Observability

### 1. **Application Metrics**

**Prometheus Metrics Collection:**
```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

pub struct Metrics {
    // Request metrics
    requests_total: Counter,
    request_duration: Histogram,
    requests_in_flight: Gauge,
    
    // Business metrics
    sync_operations_total: Counter,
    sync_duration: Histogram,
    active_syncs: Gauge,
    
    // Storage metrics
    storage_operations_total: Counter,
    storage_duration: Histogram,
    storage_usage_bytes: Gauge,
    
    // Error metrics
    errors_total: Counter,
    error_rate: Gauge,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, Error> {
        let requests_total = Counter::new(
            "leptos_sync_requests_total",
            "Total number of requests"
        )?;
        
        let request_duration = Histogram::new(
            "leptos_sync_request_duration_seconds",
            "Request duration in seconds"
        )?;
        
        let requests_in_flight = Gauge::new(
            "leptos_sync_requests_in_flight",
            "Number of requests currently being processed"
        )?;
        
        // Register metrics
        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(requests_in_flight.clone()))?;
        
        Ok(Self {
            requests_total,
            request_duration,
            requests_in_flight,
            // ... other metrics
        })
    }
    
    pub fn record_request(&self, duration: Duration) {
        self.requests_total.inc();
        self.request_duration.observe(duration.as_secs_f64());
    }
}
```

### 2. **Health Checks**

**Application Health Endpoints:**
```rust
use axum::{Router, routing::get, Json};
use serde_json::json;

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics_endpoint))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "unknown".to_string())
    }))
}

async fn readiness_check() -> Json<serde_json::Value> {
    // Check database connectivity
    let db_healthy = check_database_health().await;
    
    // Check Redis connectivity
    let redis_healthy = check_redis_health().await;
    
    let status = if db_healthy && redis_healthy {
        "ready"
    } else {
        "not_ready"
    };
    
    Json(json!({
        "status": status,
        "database": db_healthy,
        "redis": redis_healthy,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn metrics_endpoint() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

## Performance Tuning

### 1. **Database Optimization**

**Connection Pooling and Query Optimization:**
```rust
use sqlx::{PgPool, PgPoolOptions};

pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(database_url)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        // Create indexes for performance
        self.create_performance_indexes(&pool).await?;
        
        Ok(Self { pool })
    }
    
    async fn create_performance_indexes(&self, pool: &PgPool) -> Result<(), Error> {
        // Create indexes for common queries
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_sync_operations_user_id ON sync_operations(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_sync_operations_timestamp ON sync_operations(timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_sync_operations_collection ON sync_operations(collection)",
        ];
        
        for index_sql in indexes {
            sqlx::query(index_sql).execute(pool).await?;
        }
        
        Ok(())
    }
}
```

### 2. **Caching Strategy**

**Multi-Level Caching:**
```rust
use redis::AsyncCommands;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct CacheManager {
    redis: redis::Client,
    local_cache: Arc<RwLock<HashMap<String, CachedItem>>>,
}

impl CacheManager {
    pub fn new(redis_url: &str) -> Result<Self, Error> {
        let redis = redis::Client::open(redis_url)?;
        
        Ok(Self {
            redis,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        // 1. Check local cache first
        if let Some(item) = self.local_cache.read().await.get(key) {
            if !item.is_expired() {
                return Ok(Some(item.deserialize()?));
            }
        }
        
        // 2. Check Redis cache
        let mut conn = self.redis.get_async_connection().await?;
        let cached: Option<String> = conn.get(key).await?;
        
        if let Some(cached_data) = cached {
            let item: T = serde_json::from_str(&cached_data)?;
            
            // Update local cache
            let cached_item = CachedItem::new(serde_json::to_string(&item)?);
            self.local_cache.write().await.insert(key.to_string(), cached_item);
            
            return Ok(Some(item));
        }
        
        Ok(None)
    }
}
```

## Backup & Disaster Recovery

### 1. **Backup Strategy**

**Automated Backup System:**
```bash
#!/bin/bash
# scripts/backup.sh

set -e

BACKUP_DIR="/backups"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

echo "🔄 Starting backup process..."

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Database backup
echo "💾 Backing up database..."
pg_dump $DATABASE_URL | gzip > "$BACKUP_DIR/db_backup_$DATE.sql.gz"

# Redis backup
echo "💾 Backing up Redis..."
redis-cli --rdb "$BACKUP_DIR/redis_backup_$DATE.rdb"

# Configuration backup
echo "💾 Backing up configuration..."
tar -czf "$BACKUP_DIR/config_backup_$DATE.tar.gz" /etc/leptos-sync/

# Upload to S3
echo "☁️ Uploading backup to S3..."
aws s3 cp "$BACKUP_DIR/db_backup_$DATE.sql.gz" "s3://leptos-sync-backups/database/"
aws s3 cp "$BACKUP_DIR/redis_backup_$DATE.rdb" "s3://leptos-sync-backups/redis/"
aws s3 cp "$BACKUP_DIR/config_backup_$DATE.tar.gz" "s3://leptos-sync-backups/config/"

# Cleanup old backups
echo "🧹 Cleaning up old backups..."
find "$BACKUP_DIR" -type f -mtime +$RETENTION_DAYS -delete

echo "✅ Backup completed successfully!"
```

### 2. **Disaster Recovery Plan**

**Recovery Procedures:**
```bash
#!/bin/bash
# scripts/disaster-recovery.sh

set -e

RECOVERY_TYPE=${1:-full}
BACKUP_DATE=${2:-latest}

echo "🚨 Starting disaster recovery process..."
echo "Recovery type: $RECOVERY_TYPE"
echo "Backup date: $BACKUP_DATE"

case $RECOVERY_TYPE in
    "database")
        echo "🔄 Recovering database..."
        aws s3 cp "s3://leptos-sync-backups/database/db_backup_$BACKUP_DATE.sql.gz" .
        gunzip "db_backup_$BACKUP_DATE.sql.gz"
        psql $DATABASE_URL < "db_backup_$BACKUP_DATE.sql"
        ;;
    
    "redis")
        echo "🔄 Recovering Redis..."
        aws s3 cp "s3://leptos-sync-backups/redis/redis_backup_$BACKUP_DATE.rdb" .
        redis-cli shutdown
        cp "redis_backup_$BACKUP_DATE.rdb" /var/lib/redis/dump.rdb
        systemctl start redis
        ;;
    
    "full")
        echo "🔄 Performing full recovery..."
        ./scripts/disaster-recovery.sh database $BACKUP_DATE
        ./scripts/disaster-recovery.sh redis $BACKUP_DATE
        ./scripts/disaster-recovery.sh config $BACKUP_DATE
        ;;
    
    *)
        echo "❌ Unknown recovery type: $RECOVERY_TYPE"
        exit 1
        ;;
esac

echo "✅ Recovery completed successfully!"
```

## Conclusion

This comprehensive deployment and operations guide ensures that Leptos-Sync can be deployed reliably in production environments with proper monitoring, scaling, and disaster recovery capabilities.

The operations framework covers:
- ✅ Multi-environment deployment strategy
- ✅ Infrastructure as Code with Terraform
- ✅ Automated CI/CD pipelines
- ✅ Kubernetes deployment and scaling
- ✅ Health monitoring and metrics
- ✅ Performance optimization and caching
- ✅ Automated backup and disaster recovery

This foundation enables reliable production operations while maintaining performance and scalability throughout the application lifecycle.