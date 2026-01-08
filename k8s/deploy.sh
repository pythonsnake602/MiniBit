#!/bin/bash
#
# MiniBit Kubernetes Deployment Script
# 
# This script automates the deployment of MiniBit to Kubernetes
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    print_error "kubectl is not installed. Please install it first."
    exit 1
fi

# Check if we can access the cluster
if ! kubectl cluster-info &> /dev/null; then
    print_error "Cannot access Kubernetes cluster. Please configure kubectl."
    exit 1
fi

print_info "Starting MiniBit Kubernetes deployment..."

# Check if secret has been customized
if grep -q "CHANGE_ME_PLEASE" k8s/secret.yaml; then
    print_error "Please update the secret in k8s/secret.yaml before deploying!"
    print_info "Generate a secret with: openssl rand -base64 12"
    exit 1
fi

# Create namespace
print_info "Creating namespace..."
kubectl apply -f k8s/namespace.yaml

# Create secret
print_info "Creating secret..."
kubectl apply -f k8s/secret.yaml

# Create PVC
print_info "Creating persistent volume claim..."
kubectl apply -f k8s/minibit-pvc.yaml

# Wait for PVC to be bound
print_info "Waiting for PVC to be bound (this may take a moment)..."
kubectl wait --for=jsonpath='{.status.phase}'=Bound --timeout=120s pvc/minibit-config-pvc -n minibit || print_warn "PVC not bound yet, continuing anyway..."

# Create ConfigMaps
print_info "Creating Velocity configuration..."
kubectl apply -f k8s/velocity-config.yaml

# Initialize MiniBit configuration
print_info "Initializing MiniBit configuration..."
kubectl apply -f k8s/minibit-init-job.yaml

# Wait for init job to complete
print_info "Waiting for initialization to complete..."
if kubectl wait --for=condition=complete --timeout=300s job/minibit-init -n minibit 2>/dev/null; then
    print_info "Initialization completed successfully"
else
    print_warn "Init job did not complete in time or failed. Check with: kubectl logs -n minibit job/minibit-init"
fi

# Deploy MiniBit server
print_info "Deploying MiniBit server..."
kubectl apply -f k8s/minibit-deployment.yaml
kubectl apply -f k8s/minibit-service.yaml

# Deploy Velocity proxy
print_info "Deploying Velocity proxy..."
kubectl apply -f k8s/velocity-deployment.yaml
kubectl apply -f k8s/velocity-service.yaml

# Wait for deployments to be ready
print_info "Waiting for MiniBit deployment to be ready..."
kubectl wait --for=condition=available --timeout=180s deployment/minibit -n minibit || print_warn "MiniBit deployment not ready yet"

print_info "Waiting for Velocity deployment to be ready..."
kubectl wait --for=condition=available --timeout=180s deployment/velocity-proxy -n minibit || print_warn "Velocity deployment not ready yet"

# Display status
print_info "Deployment complete! Current status:"
echo ""
kubectl get all -n minibit

# Show connection information
echo ""
print_info "Connection information:"

SERVICE_TYPE=$(kubectl get svc velocity-service -n minibit -o jsonpath='{.spec.type}')

if [ "$SERVICE_TYPE" = "LoadBalancer" ]; then
    print_info "Waiting for LoadBalancer IP..."
    EXTERNAL_IP=""
    for i in {1..30}; do
        EXTERNAL_IP=$(kubectl get svc velocity-service -n minibit -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "")
        if [ -n "$EXTERNAL_IP" ]; then
            break
        fi
        sleep 2
    done
    
    if [ -n "$EXTERNAL_IP" ]; then
        echo -e "${GREEN}Connect to your server at: ${EXTERNAL_IP}:25565${NC}"
    else
        print_warn "LoadBalancer IP not assigned yet. Check with: kubectl get svc velocity-service -n minibit"
    fi
elif [ "$SERVICE_TYPE" = "NodePort" ]; then
    NODE_PORT=$(kubectl get svc velocity-service -n minibit -o jsonpath='{.spec.ports[0].nodePort}')
    print_info "Service is using NodePort"
    echo -e "${GREEN}Connect to your server at: <node-ip>:${NODE_PORT}${NC}"
    print_info "Replace <node-ip> with any of your cluster node IPs"
else
    print_warn "Service type is $SERVICE_TYPE. You may need to set up port forwarding or ingress."
fi

echo ""
print_info "To view logs:"
echo "  Velocity: kubectl logs -n minibit -l app=velocity-proxy -f"
echo "  MiniBit:  kubectl logs -n minibit -l app=minibit -f"

echo ""
print_info "To delete the deployment:"
echo "  kubectl delete namespace minibit"

print_info "Deployment script completed!"
